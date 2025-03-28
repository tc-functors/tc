use aws::Env;
use compiler::Topology;
use compiler::spec::BuildKind;
use ci::github;
use configurator::Config;
use kit as u;
use std::panic;
use std::time::Instant;
use std::str::FromStr;
use tabled::{Style, Table};

pub struct BuildOpts {
    pub merge: bool,
    pub recursive: bool,
    pub clean: bool,
    pub split: bool,
    pub dirty: bool,
}

pub async fn build(kind: Option<String>, name: Option<String>, dir: &str, opts: BuildOpts) {
    let BuildOpts {
        clean,
        dirty,
        recursive,
        ..
    } = opts;

    if recursive {

        let kind = match kind {
            Some(s) => Some(BuildKind::from_str(&s).unwrap()),
            None => None
        };
        let builds = builder::build_recursive(dirty, kind).await;
        builder::write_manifest(&builds);
        println!("{}", kit::pretty_json(&builds));


    } else if clean {
        builder::clean_lang(dir);

    } else {
        let kind = match kind {
            Some(s) => Some(BuildKind::from_str(&s).unwrap()),
            None => None
        };
        let builds = builder::build(dir, name, kind).await;
        builder::write_manifest(&builds);
        println!("{}", kit::pretty_json(&builds));
    }

}

pub struct PublishOpts {
    pub promote: bool,
    pub demote: bool,
    pub version: Option<String>,
 }

pub async fn publish(
    env: Env,
    name: Option<String>,
    dir: &str,
    opts: PublishOpts,
) {

    let PublishOpts {
        promote,
        demote,
        version,
        ..
    } = opts;

    if promote {
        let lang = &compiler::guess_runtime(&dir);
        let bname = u::maybe_string(name, u::basedir(&u::pwd()));
        publisher::promote(&env, &bname, &lang.to_str(), version).await;

    } else if demote {
        let lang = "python3.10";
        publisher::demote(&env, name, &lang).await;

    } else {
        let builds = builder::read_manifest();

        for build in builds {
            let bname = u::maybe_string(name.clone(), &build.name);
            publisher::publish(&env, &build.dir, &build.kind, &build.zipfile, &build.runtime, &bname).await;
        }
        builder::delete_manifest(dir);
    }
}

pub async fn list_published_assets(env: Env, kind: Option<String>) {
    let k = u::maybe_string(kind, "layer");
    let kind = BuildKind::from_str(&k).unwrap();
    publisher::list(&env, &kind).await
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
            let topology = compiler::compile(&dir, recursive);
            match std::env::var("TC_TRACE") {
                Ok(_) => {
                    kit::write_str("topology.json", &topology.to_str());
                    tracing::debug!("Wrote topology.json");
                    String::from("")
                }
                Err(_) => u::pretty_json(topology)
            }
        }
    }
}

pub async fn resolve(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
    no_cache: bool
) -> String {
    let topology = compiler::compile(&u::pwd(), recursive);
    let sandbox = resolver::maybe_sandbox(sandbox);
    let resolved_topology = match component.clone() {
        Some(c) => resolver::resolve_component(&env, &sandbox, &topology, &c).await,
        None => resolver::resolve(&env, &sandbox, &topology, !no_cache).await
    };

    resolver::pprint(&resolved_topology, component)
}

async fn run_create_hook(env: &Env, root: &Topology) {
    let Topology { namespace, sandbox, version, .. } = root;
    let dir = u::pwd();
    let tag = format!("{}-{}", namespace, version);
    let msg = format!(
        "Deployed `{}` to *{}*::{}_{}",
        tag, &env.name, namespace, &sandbox
    );
    notifier::notify(&namespace, &msg).await;
    if env.config.ci.update_metadata {
        let centralized = env.inherit(env.config.aws.lambda.layers_profile.to_owned());
        ci::update_metadata(&centralized, &sandbox, &namespace, &version, &env.name, &dir).await;
    }
}

async fn create_topology(env: &Env, topology: &Topology) {
    let Topology { functions, .. } = topology;

    for (_, function) in functions {
        let dir = &function.dir;
        builder::build(dir, None, None).await;
    }
    deployer::create(env, topology).await;

    for (_, node) in &topology.nodes {

        for (_, function) in &node.functions {
            let dir = &function.dir;
            builder::build(dir, None, None).await;
        }

        deployer::create(env, node).await;
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
            },
            None => kit::read_stdin()
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
    topology_path: Option<String>
) {

    let start = Instant::now();

    let maybe_topology = read_topology(topology_path).await;

    let topology = match maybe_topology {
        Some(t) => t,
        None => {
            let env = init(profile, None).await;
            let sandbox = resolver::maybe_sandbox(sandbox);
            router::guard(&sandbox);
            let dir = u::pwd();
            println!("Compiling topology");
            let ct = compiler::compile(&dir, recursive);
            let rt = resolver::resolve(&env, &sandbox, &ct, !no_cache).await;
            rt
        }
    };

    let env = init(Some(topology.env.to_string()), None).await;
    compiler::count_of(&topology);
    create_topology(&env, &topology).await;

    builder::clean(recursive);

    if notify {
        run_create_hook(&env, &topology).await;
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

async fn update_topology(env: &Env, topology: &Topology) {
    let Topology { functions, .. } = topology;

    for (_, function) in functions {
        let dir = &function.dir;
        builder::build(dir, None, None).await;
    }

    deployer::update(env, topology).await;
}

pub async fn update(env: Env, sandbox: Option<String>, recursive: bool, no_cache: bool) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    router::guard(&sandbox);
    let start = Instant::now();

    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);

    let root = resolver::resolve(&env, &sandbox, &topology, !no_cache).await;
    update_topology(&env, &root).await;

    for (_, node) in root.nodes {
        update_topology(&env, &node).await;
    }
    builder::clean(recursive);
    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

pub async fn update_component(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    router::guard(&sandbox);
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);

    let c = deployer::maybe_component(component.clone());
    let root = resolver::resolve_component(&env, &sandbox, &topology, &c).await;
    deployer::update_component(&env, &root, component.clone()).await;

    for (_, node) in root.nodes {
        deployer::update_component(&env, &node, component.clone()).await;
    }
}

pub async fn delete(env: Env, sandbox: Option<String>, recursive: bool) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    router::guard(&sandbox);
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);
    let root = resolver::resolve(&env, &sandbox, &topology, true).await;

    deployer::delete(&env, &root).await;
    for (_, node) in root.nodes {
        deployer::delete(&env, &node).await
    }
}

pub async fn delete_component(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    recursive: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    router::guard(&sandbox);
    println!("Compiling topology");
    let topology = compiler::compile(&u::pwd(), recursive);

    compiler::count_of(&topology);
    println!("Resolving topology");
    let root = resolver::resolve(&env, &sandbox, &topology, true).await;
    deployer::delete_component(&env, root.clone(), component.clone()).await;

    for (_, node) in root.nodes {
        deployer::delete_component(&env, node, component.clone()).await
    }
}

pub async fn list(
    env: Env,
    sandbox: Option<String>,
    component: Option<String>,
    format: Option<String>,
) {
    if u::option_exists(component.clone()) {
        lister::list_component(&env, sandbox, component, format).await;
    } else {
        lister::list(&env, sandbox).await;
    }
}

pub async fn scaffold() {
    let dir = u::pwd();
    let kind = compiler::kind_of();
    match kind.as_ref() {
        "function" => {
            let function = compiler::current_function(&dir);
            match function {
                Some(f) => scaffolder::create_function(&f.name, &f.dir).await,
                None => panic!("No function found"),
            }
        }
        "step-function" => {
            let functions = compiler::just_functions();
            for (_, f) in functions {
                scaffolder::create_function(&f.name, &f.dir).await;
            }
        }
        _ => {
            let function = compiler::current_function(&dir);
            match function {
                Some(f) => scaffolder::create_function(&f.name, &f.dir).await,
                None => panic!("No function found"),
            }
        }
    }
}

pub async fn bootstrap(
    env: Env,
    role_name: Option<String>,
    create: bool,
    delete: bool,
    show: bool,
) {
    match role_name {
        Some(role) => {
            if create {
                aws::bootstrap::create_role(&env, &role).await;
            } else if delete {
                aws::bootstrap::delete_role(&env, &role).await;
            } else if show {
                aws::bootstrap::show_role(&env, &role).await;
            } else {
                aws::bootstrap::show_role(&env, &role).await;
            }
        }
        None => println!("No role name given"),
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

pub async fn invoke(env: Env, opts: InvokeOptions) {
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
        let resolved = resolver::render(&env, &sandbox, &topology).await;

        let mode = match topology.flow {
            Some(f) => f.mode,
            None => "Standard".to_string()
        };
        invoker::invoke(&env, topology.kind, &resolved.fqn, payload, &mode, dumb).await;
    }
}

pub async fn emulate(env: Env, dev: bool, shell: bool) {
    let kind = compiler::kind_of();
    match kind.as_ref() {
        "step-function" => emulator::sfn().await,
        "function" => {
            if shell {
                emulator::shell(&env, dev).await;
            } else {
                emulator::lambda(&env, dev).await;
            }
        }
        _ => emulator::lambda(&env, dev).await,
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
        None => panic!("No prefix given")
    };
    let next = u::maybe_string(next, "patch");
    let suffix = u::maybe_string(suffix, "default");
    tagger::create_tag(&next, &prefix, &suffix, push, dry_run).await
}

pub async fn route(
    env: Env,
    event: Option<String>,
    service: String,
    sandbox: Option<String>,
    rule: Option<String>,
) {
    let event = u::maybe_string(event, "default");
    let sandbox = resolver::maybe_sandbox(sandbox);
    match rule {
        Some(r) => router::route(&env, &event, &service, &sandbox, &r).await,
        None => println!("Rule not specified")
    }
}

pub async fn freeze(env: Env, service: Option<String>, sandbox: String) {
    let service = u::maybe_string(service, &compiler::topology_name(&u::pwd()));
    let name = format!("{}_{}", &service, &sandbox);
    router::freeze(&env, &name).await;
    let msg = format!("*{}*::{} is frozen", &env.name, sandbox);
    notifier::notify(&service, &msg).await
}

pub async fn unfreeze(env: Env, service: Option<String>, sandbox: String) {
    let service = u::maybe_string(service, &compiler::topology_name(&u::pwd()));
    let name = format!("{}_{}", &service, &sandbox);
    router::unfreeze(&env, &name).await;
    let msg = format!("{} is now unfrozen", &name);
    notifier::notify(&service, &msg).await;
}

pub async fn upgrade(version: Option<String>) {
    github::self_upgrade("tc", "", version).await
}

// ci

pub async fn deploy(
    service: Option<String>,
    env: String,
    sandbox: Option<String>,
    version: String
) {
    let dir = u::pwd();
    let namespace = compiler::topology_name(&dir);
    let service = u::maybe_string(service, &namespace);
    let sandbox = u::maybe_string(sandbox, "stable");
    ci::deploy(&env, &service, &sandbox, &version).await;
}

pub async fn release(
    service: Option<String>,
    suffix: Option<String>,
    unwind: bool,
) {
    let dir = u::pwd();
    let suffix = u::maybe_string(suffix, "default");
    let namespace = compiler::topology_name(&dir);
    let service = u::maybe_string(service, &namespace);
    if unwind {
        ci::unwind(&service);
    } else {
        ci::release(&service, &suffix).await
    }
}

pub async fn show_config() {
    let config = Config::new(None, "dev");
    println!("{}", config.render());
}

pub async fn download_layer(env: Env, name: Option<String>) {
    match name {
        Some(n) => publisher::download_layer(&env, &n).await,
        None => println!("provide layer name")
    }
}

pub async fn init(profile: Option<String>, assume_role: Option<String>) -> Env {
    match profile {
        Some(ref p) => aws::init(profile.clone(), assume_role, Config::new(None, &p)).await,
        None => aws::init(profile, assume_role, Config::new(None, "")).await
    }
}

pub async fn init_repo_profile(profile: Option<String>) -> Env {
    match profile {
        Some(ref p) => aws::init(profile.clone(), None, Config::new(None, &p)).await,
        None => {
            let given_env = aws::init(profile, None, Config::new(None, "")).await;
            given_env.inherit(given_env.config.aws.lambda.layers_profile.to_owned())
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
        },
        None => {
            let xs = resolver::cache::list();
            let table = Table::new(xs).with(Style::psql()).to_string();
            println!("{}", table);
        }
    }
}

pub async fn inspect() {
    inspector::init().await
}
