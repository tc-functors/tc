use compiler::{
    Topology,
    spec::{
        BuildKind,
        BuildSpec,
        ConfigSpec,
        FunctionSpec,
        function::InfraSpec,
    },
};
use kit as u;
use authorizer::Auth;
use std::{
    panic,
    str::FromStr,
    time::Instant,
};
use tabled::{
    Style,
    Table,
};

pub struct BuildOpts {
    pub recursive: bool,
    pub clean: bool,
    pub dirty: bool,
    pub publish: bool,
    pub sync: bool,
    pub shell: bool,
    pub image: Option<String>,
    pub lang: Option<String>,
}

pub async fn build(
    profile: Option<String>,
    kind: Option<String>,
    name: Option<String>,
    dir: &str,
    opts: BuildOpts,
) {
    let BuildOpts {
        clean,
        dirty,
        recursive,
        image,
        sync,
        publish,
        lang,
        shell,
        ..
    } = opts;

    if recursive {
        let kind = match kind {
            Some(s) => Some(BuildKind::from_str(&s).unwrap()),
            None => None,
        };
        if sync {
            let auth = init(profile, None).await;
            let builds = builder::just_images(recursive);
            builder::sync(&auth, builds).await;
        } else {
            let builds = builder::build_recursive(dirty, kind, image).await;
            if publish {
                let auth = init(profile.clone(), None).await;
                builder::publish(&auth, builds.clone()).await;
            }
        }

    } else if clean {
        builder::clean_lang(dir);
    } else {
        let kind = match kind {
            Some(s) => Some(BuildKind::from_str(&s).unwrap()),
            None => None,
        };
        if sync {
            let auth = init(profile, None).await;
            let builds = builder::just_images(false);
            builder::sync(&auth, builds).await;
        } else if shell {
            builder::shell(dir);
        } else {
            let builds = builder::build(dir, name, kind, image, lang).await;
            if publish {
                let auth = init(profile.clone(), None).await;
                builder::publish(&auth, builds.clone()).await;
            }
        }
    }
}

pub async fn promote(auth: Auth, name: Option<String>, version: Option<String>) {
    let dir = &u::pwd();
    builder::promote(&auth, name, dir, version).await;
}

pub async fn test() {
    let dir = u::pwd();
    let spec = compiler::compile(&dir, false);
    for (path, function) in spec.functions {
        tester::test(&path, function).await;
    }
}

pub struct CompileOpts {
    pub versions: bool,
    pub recursive: bool,
    pub component: Option<String>,
    pub format: Option<String>,
}

pub async fn compile(opts: CompileOpts) -> String {
    let CompileOpts {
        recursive,
        component,
        format,
        ..
    } = opts;

    let dir = u::pwd();
    let format = u::maybe_string(format, "json");

    match component {
        Some(c) => compiler::show_component(&c, &format, recursive),
        None => {
            if compiler::is_root_dir(&dir) {
                let res = compiler::compile_root(&dir, recursive);
                compiler::formatter::print_topologies(res);
                String::from("")
            } else {
                let topology = compiler::compile(&dir, recursive);
                match std::env::var("TC_TRACE") {
                    Ok(_) => {
                        kit::write_str("topology.json", &topology.to_str());
                        tracing::debug!("Wrote topology.json");
                        String::from("")
                    }
                    Err(_) => u::pretty_json(topology),
                }
            }
        }
    }
}

pub async fn resolve(
    auth: Auth,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
    no_cache: bool,
) -> String {

    let topology = compiler::compile(&u::pwd(), recursive);
    let sandbox = resolver::maybe_sandbox(sandbox);
    let resolved_topology = match component.clone() {
        Some(c) => resolver::resolve_component(&auth, &sandbox, &topology, &c).await,
        None => resolver::resolve(&auth, &sandbox, &topology, !no_cache).await,
    };

    resolver::pprint(&resolved_topology, component)
}

async fn run_create_hook(auth: &Auth, root: &Topology) {
    let Topology {
        namespace,
        sandbox,
        version,
        config,
        ..
    } = root;
    let dir = u::pwd();
    let tag = format!("{}-{}", namespace, version);
    let msg = format!(
        "Deployed `{}` to *{}*::{}_{}",
        tag, &auth.name, namespace, &sandbox
    );
    releaser::notify(&namespace, &msg).await;
    if config.ci.update_metadata {
        let profile = config.aws.lambda.layers_profile.clone();
        let centralized = auth.assume(profile.clone(), config.role_to_assume(profile)).await;
        releaser::ci::update_metadata(
            &centralized,
            &sandbox,
            &namespace,
            &version,
            &auth.name,
            &dir,
        )
        .await;
    }
}

async fn maybe_build(auth: &Auth, dir: &str, name: &str) {
    let builds = builder::build(
        dir,
        Some(String::from(name)),
        None,
        Some(String::from("code")),
        None,
    )
    .await;
    let config = ConfigSpec::new(None);
    let profile = config.aws.lambda.layers_profile.clone();
    let centralized = auth.assume(profile.clone(), config.role_to_assume(profile)).await;
    builder::publish(&centralized, builds).await;
}

async fn create_topology(auth: &Auth, topology: &Topology) {
    let Topology { functions, .. } = topology;

    for (_, function) in functions {
        maybe_build(auth, &function.dir, &function.name).await;
    }
    deployer::create(auth, topology).await;

    for (_, node) in &topology.nodes {
        for (_, function) in &node.functions {
            let dir = &function.dir;
            maybe_build(auth, dir, &function.name).await;
        }

        deployer::create(auth, node).await;
    }
}

async fn read_topology(path: Option<String>) -> Option<Topology> {
    if u::option_exists(path.clone()) {
        let data = match path {
            Some(p) => {
                if kit::file_exists(&p) {
                    kit::slurp(&p)
                } else {
                    kit::read_stdin()
                }
            }
            None => kit::read_stdin(),
        };
        let t: Topology = serde_json::from_str(&data).unwrap();
        Some(t)
    } else {
        None
    }
}

pub async fn create(
    profile: Option<String>,
    sandbox: Option<String>,
    notify: bool,
    recursive: bool,
    no_cache: bool,
    topology_path: Option<String>,
) {
    let start = Instant::now();

    let maybe_topology = read_topology(topology_path).await;

    let topology = match maybe_topology {
        Some(t) => t,
        None => {
            let auth = init(profile, None).await;
            let sandbox = resolver::maybe_sandbox(sandbox);
            releaser::guard(&sandbox);
            let dir = u::pwd();
            println!("Compiling topology");
            let ct = compiler::compile(&dir, recursive);
            let rt = resolver::resolve(&auth, &sandbox, &ct, !no_cache).await;
            rt
        }
    };

    let auth = init(Some(topology.env.to_string()), None).await;
    let msg = compiler::count_of(&topology);
    println!("{}", msg);
    create_topology(&auth, &topology).await;

    match std::env::var("TC_INSPECT_BUILD") {
        Ok(_) => (),
        Err(_) => builder::clean(recursive),
    }

    if notify {
        run_create_hook(&auth, &topology).await;
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

async fn update_topology(auth: &Auth, topology: &Topology) {
    let Topology { functions, .. } = topology;

    for (_, function) in functions {
        maybe_build(auth, &function.dir, &function.name).await;
    }

    deployer::update(auth, topology).await;
}

pub async fn update(auth: Auth, sandbox: Option<String>, recursive: bool, no_cache: bool) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    releaser::guard(&sandbox);
    let start = Instant::now();

    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);

    let root = resolver::resolve(&auth, &sandbox, &topology, !no_cache).await;
    update_topology(&auth, &root).await;

    for (_, node) in root.nodes {
        update_topology(&auth, &node).await;
    }
    builder::clean(recursive);
    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

pub async fn update_component(
    auth: Auth,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    releaser::guard(&sandbox);
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);

    let c = deployer::maybe_component(component.clone());
    let root = resolver::resolve_component(&auth, &sandbox, &topology, &c).await;
    deployer::update_component(&auth, &root, component.clone()).await;

    for (_, node) in root.nodes {
        deployer::update_component(&auth, &node, component.clone()).await;
    }
}

pub async fn delete(auth: Auth, sandbox: Option<String>, recursive: bool) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    releaser::guard(&sandbox);
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);
    let root = resolver::resolve(&auth, &sandbox, &topology, true).await;

    deployer::delete(&auth, &root).await;
    for (_, node) in root.nodes {
        deployer::delete(&auth, &node).await
    }
}

pub async fn delete_component(
    auth: Auth,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    releaser::guard(&sandbox);
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);
    println!("Resolving topology");
    let root = resolver::resolve(&auth, &sandbox, &topology, true).await;
    deployer::delete_component(&auth, root.clone(), component.clone()).await;

    for (_, node) in root.nodes {
        deployer::delete_component(&auth, node, component.clone()).await
    }
}

pub async fn list(
    auth: Auth,
    sandbox: Option<String>,
    component: Option<String>,
    format: Option<String>,
) {
    if u::option_exists(component.clone()) {
        differ::list_component(&auth, sandbox, component, format).await;
    } else {
        differ::list(&auth, sandbox).await;
    }
}

pub struct InvokeOptions {
    pub sandbox: Option<String>,
    pub payload: Option<String>,
    pub name: Option<String>,
    pub kind: Option<String>,
    pub local: bool,
    pub dumb: bool,
}

pub async fn invoke(auth: Auth, opts: InvokeOptions) {
    let InvokeOptions {
        sandbox,
        payload,
        local,
        dumb,
        ..
    } = opts;

    if local {
        invoker::run_local(payload).await;
    } else {
        // FIXME: get dir
        let topology = compiler::compile(&u::pwd(), false);

        let sandbox = resolver::maybe_sandbox(sandbox);
        let resolved = resolver::render(&auth, &sandbox, &topology).await;

        let mode = match topology.flow {
            Some(f) => f.mode,
            None => "Standard".to_string(),
        };
        invoker::invoke(&auth, topology.kind, &resolved.fqn, payload, &mode, dumb).await;
    }
}

pub async fn emulate(auth: Auth, dev: bool, shell: bool) {
    let kind = compiler::kind_of();
    match kind.as_ref() {
        "step-function" => emulator::sfn().await,
        "function" => {
            if shell {
                emulator::shell(&auth, dev).await;
            } else {
                emulator::lambda(&auth, dev).await;
            }
        }
        _ => emulator::lambda(&auth, dev).await,
    }
}

pub async fn tag(
    prefix: Option<String>,
    next: Option<String>,
    dry_run: bool,
    push: bool,
    suffix: Option<String>,
) {
    let prefix = match prefix {
        Some(p) => p,
        None => panic!("No prefix given"),
    };
    let next = u::maybe_string(next, "patch");
    let suffix = u::maybe_string(suffix, "default");
    releaser::create_tag(&next, &prefix, &suffix, push, dry_run).await
}

pub async fn route(
    auth: Auth,
    event: Option<String>,
    service: String,
    sandbox: Option<String>,
    rule: Option<String>,
) {
    let event = u::maybe_string(event, "default");
    let sandbox = resolver::maybe_sandbox(sandbox);
    match rule {
        Some(r) => releaser::route(&auth, &event, &service, &sandbox, &r).await,
        None => println!("Rule not specified"),
    }
}

pub async fn freeze(auth: Auth, service: Option<String>, sandbox: String) {
    let service = u::maybe_string(service, &compiler::topology_name(&u::pwd()));
    let name = format!("{}_{}", &service, &sandbox);
    releaser::freeze(&auth, &name).await;
    let msg = format!("*{}*::{} is frozen", &auth.name, sandbox);
    releaser::notify(&service, &msg).await
}

pub async fn unfreeze(auth: Auth, service: Option<String>, sandbox: String) {
    let service = u::maybe_string(service, &compiler::topology_name(&u::pwd()));
    let name = format!("{}_{}", &service, &sandbox);
    releaser::unfreeze(&auth, &name).await;
    let msg = format!("{} is now unfrozen", &name);
    releaser::notify(&service, &msg).await;
}

pub async fn upgrade(version: Option<String>) {
    releaser::self_upgrade("tc", version).await
}

// ci
// deprecated

pub async fn ci_deploy(
    service: Option<String>,
    env: String,
    sandbox: Option<String>,
    version: String,
) {
    let dir = u::pwd();
    let namespace = compiler::topology_name(&dir);
    let service = u::maybe_string(service, &namespace);
    let sandbox = u::maybe_string(sandbox, "stable");
    releaser::ci::deploy(&env, &service, &sandbox, &version).await;
}

pub async fn ci_release(service: Option<String>, suffix: Option<String>, unwind: bool) {
    let dir = u::pwd();
    let suffix = u::maybe_string(suffix, "default");
    let namespace = compiler::topology_name(&dir);
    let service = u::maybe_string(service, &namespace);
    if unwind {
        releaser::unwind(&service);
    } else {
        releaser::ci::release(&service, &suffix).await
    }
}

pub async fn ci_upgrade(version: Option<String>) {
    let repo = "tc";
    let maybe_release_id = releaser::get_release_id(&repo, version).await;
    match maybe_release_id {
        Some(id) => {
            releaser::ci::update_var("TC_RELEASE_ID_TEST", &id).await;
        },
        None => println!("No release id found")
    }
}

pub async fn show_config() {
    let config = ConfigSpec::new(None);
    println!("{}", config.render());
}

pub async fn init(
    profile: Option<String>,
    assume_role: Option<String>
) -> Auth {

    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            let role = match assume_role {
                Some(r) => Some(r),
                None => {
                    let config = compiler::config(&kit::pwd());
                    let p = u::maybe_string(profile.clone(), "default");
                    config.ci.roles.get(&p).cloned()
                }
            };
            Auth::new(profile.clone(), role).await
        }
        Err(_) => {
            Auth::new(profile.clone(), assume_role).await
        }
    }
}

pub async fn clear_cache() {
    resolver::cache::clear()
}

pub async fn list_cache(namespace: Option<String>, env: Option<String>, sandbox: Option<String>) {
    match namespace {
        Some(n) => {
            let env = kit::maybe_string(env, "default");
            let sandbox = kit::maybe_string(sandbox, "default");
            let key = resolver::cache::make_key(&n, &env, &sandbox);
            let topology = resolver::cache::read_topology(&key).await;
            println!("{}", kit::pretty_json(&topology));
        }
        None => {
            let xs = resolver::cache::list();
            let table = Table::new(xs).with(Style::psql()).to_string();
            println!("{}", table);
        }
    }
}

pub fn generate_doc(spec: &str) {
    let schema = match spec {
        "build" => doku::to_json::<BuildSpec>(),
        "infra" => doku::to_json::<InfraSpec>(),
        "function" => doku::to_json::<FunctionSpec>(),
        _ => doku::to_json::<FunctionSpec>(),
    };
    println!("{}", &schema);
    //println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}

pub async fn inspect(port: Option<String>) {
    inspector::init(port).await
}
