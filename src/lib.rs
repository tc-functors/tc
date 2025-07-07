use authorizer::Auth;
use composer::{
    Entity,
    Topology,
    spec::{
        BuildSpec,
        ConfigSpec,
        FunctionSpec,
        function::InfraSpec,
    },
};
use kit as u;
use std::{
    panic,
    time::Instant,
};
use tabled::{
    Style,
    Table,
};

pub struct BuildOpts {
    pub recursive: bool,
    pub parallel: bool,
    pub clean: bool,
    pub publish: bool,
    pub remote: bool,
    pub sync: bool,
    pub shell: bool,
    pub kind: Option<String>,
    pub image: Option<String>,
    pub layer: Option<String>,
}

pub async fn build(profile: Option<String>, name: Option<String>, dir: &str, opts: BuildOpts) {
    let BuildOpts {
        clean,
        recursive,
        image,
        layer,
        kind,
        sync,
        publish,
        shell,
        parallel,
        ..
    } = opts;

    if recursive {
        if sync {
            let auth = init(profile, None).await;
            let builds = builder::just_images(recursive);
            builder::sync(&auth, builds).await;
        } else {
            let builds = builder::build_recursive(dir, parallel, image, layer).await;
            if publish {
                let auth = init(profile.clone(), None).await;
                builder::publish(&auth, builds.clone()).await;
            }
        }
    } else if clean {
        builder::clean_lang(dir);
    } else {
        if sync {
            let auth = init(profile, None).await;
            let builds = builder::just_images(false);
            builder::sync(&auth, builds).await;
        } else if shell {
            builder::shell(dir);
        } else {
            let maybe_fn = composer::current_function(dir);
            match maybe_fn {
                Some(f) => {
                    let builds = builder::build(&f, name, image, layer, kind).await;
                    if publish {
                        let auth = init(profile.clone(), None).await;
                        builder::publish(&auth, builds.clone()).await;
                    }
                }
                None => println!("No function found. Try --recursive or build from a function dir"),
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
    let spec = composer::compose(&dir, false);
    for (path, function) in spec.functions {
        tester::test(&path, function).await;
    }
}

pub struct ComposeOpts {
    pub versions: bool,
    pub recursive: bool,
    pub entity: Option<String>,
    pub format: Option<String>,
}

pub async fn compose(opts: ComposeOpts) {
    let ComposeOpts {
        recursive,
        entity,
        format,
        ..
    } = opts;

    let dir = u::pwd();
    let fmt = u::maybe_string(format.clone(), "json");

    match entity {
        Some(e) => composer::display_entity(&dir, &e, &fmt, recursive),
        None => match format {
            Some(fmt) => composer::display_topology(&dir, &fmt, recursive),
            None => {
                if composer::is_root_dir(&dir) {
                    composer::display_root();
                } else {
                    let topology = composer::compose(&dir, recursive);
                    match std::env::var("TC_DUMP_TOPOLOGY") {
                        Ok(_) => {
                            kit::write_str("topology.json", &topology.to_str());
                            tracing::debug!("Wrote topology.json");
                        }
                        Err(_) => u::pp_json(topology),
                    }
                }
            }
        },
    }
}

pub async fn resolve(
    auth: Auth,
    sandbox: Option<String>,
    maybe_entity: Option<String>,
    recursive: bool,
    cache: bool,
) {
    let topology = composer::compose(&u::pwd(), recursive);
    let sandbox = resolver::maybe_sandbox(sandbox);
    let rt = resolver::try_resolve(&auth, &sandbox, &topology, &maybe_entity, cache).await;
    let entity = Entity::as_entity(maybe_entity);
    composer::pprint(&rt, entity)
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
        let centralized = auth
            .assume(profile.clone(), config.role_to_assume(profile))
            .await;
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

async fn create_topology(auth: &Auth, topology: &Topology) {
    deployer::create(auth, topology).await;

    for (_, node) in &topology.nodes {
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
    cache: bool,
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
            println!("Compiling topology {} ...", &composer::topology_name(&dir));
            let ct = composer::compose(&dir, recursive);
            println!("Resolving topology {} ...", &ct.namespace);
            let rt = resolver::resolve(&auth, &sandbox, &ct, cache).await;
            rt
        }
    };

    let auth = init(Some(topology.env.to_string()), None).await;
    let msg = composer::count_of(&topology);
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

pub async fn update(
    auth: Auth,
    sandbox: Option<String>,
    maybe_entity: Option<String>,
    recursive: bool,
    cache: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);

    releaser::guard(&sandbox);

    let start = Instant::now();

    println!("Compiling topology...");
    let topology = composer::compose(&u::pwd(), recursive);

    let msg = composer::count_of(&topology);
    println!("{}", msg);

    println!("Resolving topology {}...", &topology.namespace);
    let root = resolver::try_resolve(&auth, &sandbox, &topology, &maybe_entity, cache).await;

    deployer::try_update(&auth, &root, &maybe_entity.clone()).await;

    for (_, node) in root.nodes {
        deployer::try_update(&auth, &node, &maybe_entity).await;
    }
    builder::clean(recursive);
    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

pub async fn delete(
    auth: Auth,
    sandbox: Option<String>,
    maybe_entity: Option<String>,
    recursive: bool,
    cache: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    releaser::guard(&sandbox);

    let start = Instant::now();
    println!("Compiling topology...");
    let topology = composer::compose(&u::pwd(), recursive);

    composer::count_of(&topology);
    println!("Resolving topology...");
    let root = resolver::try_resolve(&auth, &sandbox, &topology, &maybe_entity, cache).await;

    deployer::try_delete(&auth, &root, &maybe_entity).await;

    for (_, node) in root.nodes {
        deployer::try_delete(&auth, &node, &maybe_entity).await;
    }
    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
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
        let topology = composer::compose(&u::pwd(), false);

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
    let kind = composer::kind_of();
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
    let service = u::maybe_string(service, &composer::topology_name(&u::pwd()));
    let name = format!("{}_{}", &service, &sandbox);
    releaser::freeze(&auth, &name).await;
    let msg = format!("*{}*::{} is frozen", &auth.name, sandbox);
    releaser::notify(&service, &msg).await
}

pub async fn unfreeze(auth: Auth, service: Option<String>, sandbox: String) {
    let service = u::maybe_string(service, &composer::topology_name(&u::pwd()));
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
    topology: Option<String>,
    env: String,
    sandbox: Option<String>,
    version: Option<String>,
) {
    let dir = u::pwd();
    let namespace = composer::topology_name(&dir);
    let current_version = composer::topology_version(&namespace);
    let name = u::maybe_string(topology, &namespace);
    let sandbox = u::maybe_string(sandbox, "stable");
    let version = u::maybe_string(version, &current_version);
    releaser::ci::deploy(&env, &name, &sandbox, &version).await;
}

pub async fn ci_release(service: Option<String>, suffix: Option<String>, unwind: bool) {
    let dir = u::pwd();
    let suffix = u::maybe_string(suffix, "default");
    let namespace = composer::topology_name(&dir);
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
        }
        None => println!("No release id found"),
    }
}

pub async fn show_config() {
    let config = ConfigSpec::new(None);
    println!("{}", config.render());
}

pub async fn init(profile: Option<String>, assume_role: Option<String>) -> Auth {
    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            let role = match assume_role {
                Some(r) => Some(r),
                None => {
                    let config = composer::config(&kit::pwd());
                    let p = u::maybe_string(profile.clone(), "default");
                    config.ci.roles.get(&p).cloned()
                }
            };
            Auth::new(profile.clone(), role).await
        }
        Err(_) => Auth::new(profile.clone(), assume_role).await,
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

pub async fn snapshot(
    profile: Option<String>,
    sandbox: Option<String>,
    format: Option<String>,
    manifest: bool,
    save: Option<String>,
    target_profile: Option<String>,
) {
    let dir = u::pwd();
    let format = u::maybe_string(format, "json");
    let sandbox = u::maybe_string(sandbox, "stable");

    match profile {
        Some(ref p) => {
            let profiles: Vec<String> = p.split(",").map(|v| v.to_string()).collect();
            if profiles.len() > 1 {
                snapshotter::snapshot_profiles(&dir, &sandbox, profiles).await;
            } else {
                let auth = init(profile.clone(), None).await;

                if manifest {
                    snapshotter::generate_manifest(&auth, &dir, &sandbox, save, target_profile)
                        .await;
                } else {
                    let records = snapshotter::snapshot(&auth, &dir, &sandbox).await;
                    snapshotter::pretty_print(records, &format);
                }
            }
        }
        None => println!("Please specify profile"),
    }
}

pub async fn changelog(between: Option<String>, search: Option<String>, verbose: bool) {
    let dir = u::pwd();
    let namespace = composer::topology_name(&dir);
    match search {
        Some(s) => {
            let is_root = composer::is_root_dir(&dir);
            if is_root {
                let namespaces = composer::root_namespaces(&dir);
                for (_, namespace) in namespaces {
                    let version = releaser::find_version_history(&namespace, &s).await;
                    if let Some(v) = version {
                        println!("{},{},{}", &s, &namespace, &v);
                    }
                }
            }
        }
        None => releaser::changelog(&namespace, between, verbose),
    }
}

pub async fn bootstrap(auth: &Auth) {
    deployer::base::create_roles(auth).await
}

pub async fn prune(auth: &Auth, sandbox: Option<String>, dry_run: bool) {
    match sandbox {
        Some(sbox) => {
            if dry_run {
                pruner::list(auth, &sbox).await;
            } else {
                releaser::guard(&sbox);
                pruner::prune(auth, &sbox).await;
            }
        }
        None => println!("Please specify sandbox"),
    }
}
