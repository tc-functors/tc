use compiler::Entity;
use composer::{
    Topology,
};
use configurator::Config;
use itertools::Itertools;
use kit as u;
use provider::Auth;
use std::{
    panic,
    time::Instant,
};
use tabled::{
    Style,
    Table,
};
mod interactive;

pub struct BuildOpts {
    pub recursive: bool,
    pub parallel: bool,
    pub clean: bool,
    pub publish: bool,
    pub promote: bool,
    pub sync: bool,
    pub shell: bool,
    pub kind: Option<String>,
    pub version: Option<String>,
}

async fn init_centralized_auth(maybe_profile: Option<String>) -> Auth {
    let config = Config::new();
    let maybe_cfg_profile = config.aws.lambda.layers_profile.clone();
    let profile = match maybe_cfg_profile {
        Some(p) => p,
        None => match maybe_profile {
            Some(x) => x,
            None => panic!("Please specify profile"),
        },
    };
    let prof = Some(profile);
    let auth = init(prof.clone(), None).await;
    let centralized = auth.assume(prof.clone(), config.role_to_assume(prof)).await;
    centralized
}

pub async fn build(profile: Option<String>, name: Option<String>, dir: &str, opts: BuildOpts) {
    let BuildOpts {
        clean,
        recursive,
        kind,
        sync,
        publish,
        shell,
        parallel,
        promote,
        version,
        ..
    } = opts;

    if recursive {
        let auth = init_centralized_auth(profile).await;
        if sync {
            let builds = builder::just_images(recursive);
            builder::sync(&auth, builds).await;
        } else {
            let builds = builder::build_recursive(&auth, dir, parallel).await;
            if publish {
                builder::publish(&auth, builds.clone()).await;
            }
        }
    } else if clean {
        builder::clean_lang(dir);
    } else {
        if sync {
            let builds = builder::just_images(false);
            let auth = init_centralized_auth(profile).await;
            builder::sync(&auth, builds).await;
        } else if shell {
            let auth = init_centralized_auth(profile).await;
            builder::shell(&auth, dir, kind).await;
        } else if promote {
            let auth = init_centralized_auth(profile).await;
            if let Some(n) = name {
                builder::promote(&auth, &n, dir, version).await;
            }
        } else {
            let maybe_fn = composer::current_function(dir);
            match maybe_fn {
                Some(f) => {
                    let auth = init_centralized_auth(profile).await;
                    let builds = builder::build(&auth, &f, name, kind, false).await;
                    if publish {
                        builder::publish(&auth, builds.clone()).await;
                    }
                }
                None => println!("No function found. Try --recursive or build from a function dir"),
            }
        }
    }
}

pub async fn test_interactive(auth: Auth, sandbox: Option<String>) {
    let dir = u::pwd();
    let sandbox = resolver::maybe_sandbox(sandbox);

    if composer::is_topology_dir(&dir) {
        let topology = composer::compose(&dir, false);
        let units = &topology.tests;

        let (name, maybe_unit) = interactive::prompt_test_units(units.clone());
        if let Some(spec) = maybe_unit {
            let resolved = resolver::render(&auth, &sandbox, &topology).await;
            tester::test_topology_unit(&auth, &topology.namespace, &name, &resolved, &spec).await;
        }
    } else {
        println!("Interactive mode supported only in topology directory");
    }
}

pub async fn test(auth: Auth, sandbox: Option<String>, unit: Option<String>, recursive: bool) {
    let dir = u::pwd();
    let sandbox = resolver::maybe_sandbox(sandbox);

    if composer::is_topology_dir(&dir) {
        let topology = composer::compose(&dir, recursive);
        let resolved = resolver::render(&auth, &sandbox, &topology).await;
        tester::test_topology(&auth, &resolved, unit).await;
    } else {
        if let Some(f) = composer::current_function(&dir) {
            tester::test_function(&auth, &sandbox, &f, unit).await;
        }
    }
}

pub struct ComposeOpts {
    pub versions: bool,
    pub recursive: bool,
    pub entity: Option<String>,
    pub format: Option<String>,
}

pub async fn compile(dir: Option<String>, recursive: bool) {
    let dir = u::maybe_string(dir, &u::pwd());
    let spec = compiler::compile(&dir, recursive);
    spec.pprint()
}

pub async fn compose_root(dir: Option<String>, format: Option<String>) {
    let root_dir = match dir {
        Some(d) => d,
        None => u::root(),
    };
    let fmt = u::maybe_string(format, "table");
    let tps = composer::compose_root(&root_dir, true);
    composer::print_topologies(&fmt, tps);
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
    trace: bool,
) {
    let topology = composer::compose(&u::pwd(), recursive);
    let sandbox = resolver::maybe_sandbox(sandbox);
    let rt = resolver::try_resolve(&auth, &sandbox, &topology, &maybe_entity, cache, true).await;
    if !trace {
        let entity = Entity::as_entity(maybe_entity);
        composer::pprint(&rt, entity)
    }
}

pub async fn diff(auth: Auth, sandbox: Option<String>, recursive: bool, _trace: bool) {
    let topology = composer::compose(&u::pwd(), recursive);
    let sandbox = resolver::maybe_sandbox(sandbox);

    let topology = resolver::render(&auth, &sandbox, &topology).await;
    let Topology {
        namespace,
        fqn,
        version,
        kind,
        ..
    } = topology.clone();

    let rt = resolver::function::Root {
        namespace: namespace.to_string(),
        fqn: fqn.to_string(),
        version: version.to_string(),
        kind: kind.clone(),
    };

    let functions = resolver::function::find_modified(&auth, &rt, &topology).await;
    println!("Modified functions:");
    for (name, _) in functions {
        println!("{}", name);
    }
    for (_, node) in &topology.nodes {
        let functions = resolver::function::find_modified(&auth, &rt, &node).await;
        for (name, _) in functions {
            println!("{}/{}", node.namespace, name);
        }
    }
}

pub async fn diff_between(between: &str) {
    let topology = composer::compose(&u::pwd(), true);

    let (from, to) = between.split("...").collect_tuple().unwrap();
    let fns = resolver::function::diff(&topology.namespace, &from, &to, &topology.functions);

    println!("Modified functions:");
    for (name, _) in fns {
        println!("{}", name);
    }
    for (_, node) in &topology.nodes {
        let fns = resolver::function::diff(&topology.namespace, &from, &to, &node.functions);
        for (name, _) in fns {
            println!("{}/{}", node.namespace, name);
        }
    }
}

async fn run_create_hook(auth: &Auth, root: &Topology) {
    let Topology {
        namespace,
        sandbox,
        version,
        ..
    } = root;
    let tag = format!("{}-{}", namespace, version);
    let msg = format!(
        "Deployed `{}` to *{}*::{}_{}",
        tag, &auth.name, namespace, &sandbox
    );
    notifier::notify(&namespace, &msg).await;
}

async fn create_topology(auth: &Auth, topology: &Topology, sync: bool) {
    deployer::create(auth, topology, sync).await;

    for (_, node) in &topology.nodes {
        deployer::create(auth, node, sync).await;
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
    sync: bool,
) {
    let start = Instant::now();

    let maybe_topology = read_topology(topology_path).await;

    let topology = match maybe_topology {
        Some(t) => t,
        None => {
            let auth = init(profile, None).await;
            let sandbox = resolver::maybe_sandbox(sandbox);
            deployer::guard::prevent_stable_updates(&sandbox);
            let dir = u::pwd();
            println!("Composing topology {} ...", &composer::topology_name(&dir));
            let ct = composer::compose(&dir, recursive);
            println!("Resolving topology {} ...", &ct.namespace);
            let rt = resolver::resolve(&auth, &sandbox, &ct, cache, true).await;
            rt
        }
    };

    let auth = init(Some(topology.env.to_string()), None).await;
    let msg = composer::count_of(&topology);
    println!("{}", msg);
    create_topology(&auth, &topology, sync).await;

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

async fn update_aux(auth: &Auth, sandbox: &str, topology: &Topology, maybe_entity: Option<String>) {
    let diff = false;
    let cache = false;

    let start = Instant::now();
    match maybe_entity {
        Some(ref e) => println!("Resolving {} ...", &e),
        None => println!("Resolving topology {}...", &topology.namespace),
    }

    let root = resolver::try_resolve(&auth, &sandbox, &topology, &maybe_entity, cache, diff).await;

    deployer::try_update(&auth, &root, &maybe_entity.clone()).await;

    for (_, node) in root.nodes {
        deployer::try_update(&auth, &node, &maybe_entity).await;
    }
    builder::clean(true);

    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

pub async fn update(
    auth: Auth,
    sandbox: Option<String>,
    maybe_entity: Option<String>,
    recursive: bool,
    _cache: bool,
    interactive: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);

    deployer::guard::prevent_stable_updates(&sandbox);

    println!("Composing topology...");
    let topology = composer::compose(&u::pwd(), recursive);
    let msg = composer::count_of(&topology);
    println!("{}", msg);

    let entities = composer::entities_of(&topology);

    let maybe_entity = if interactive {
        interactive::prompt_entity_components(&topology, entities)
    } else {
        maybe_entity
    };
    update_aux(&auth, &sandbox, &topology, maybe_entity).await
}

pub async fn delete(
    auth: Auth,
    sandbox: Option<String>,
    maybe_entity: Option<String>,
    recursive: bool,
    cache: bool,
) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    deployer::guard::prevent_stable_updates(&sandbox);

    let start = Instant::now();
    println!("Composing topology...");
    let topology = composer::compose(&u::pwd(), recursive);

    composer::count_of(&topology);
    println!("Resolving topology...");
    let root = resolver::try_resolve(&auth, &sandbox, &topology, &maybe_entity, cache, false).await;

    deployer::try_delete(&auth, &root, &maybe_entity).await;

    for (_, node) in root.nodes {
        deployer::try_delete(&auth, &node, &maybe_entity).await;
    }
    let duration = start.elapsed();
    println!("Time elapsed: {:#}", u::time_format(duration));
}

pub struct InvokeOptions {
    pub sandbox: Option<String>,
    pub dir: Option<String>,
    pub payload: Option<String>,
    pub entity: Option<String>,
    pub emulator: bool,
    pub dumb: bool,
}

pub async fn invoke(profile: Option<String>, opts: InvokeOptions) {
    let InvokeOptions {
        sandbox,
        payload,
        emulator,
        dumb,
        entity,
        dir,
        ..
    } = opts;

    let dir = u::maybe_string(dir, &u::pwd());

    let topology = composer::compose(&dir, false);
    let sandbox = resolver::maybe_sandbox(sandbox);

    if emulator {
        let profile = u::maybe_string(profile, "dev");
        let auth = init(Some(profile), None).await;
        let resolved = resolver::render(&auth, &sandbox, &topology).await;
        invoker::invoke_emulator(&auth, entity, &resolved, payload).await;
    } else {
        let auth = init(profile, None).await;
        let resolved = resolver::render(&auth, &sandbox, &topology).await;
        invoker::invoke(&auth, entity, &resolved, payload, dumb).await;
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
    tagger::create_tag(&next, &prefix, &suffix, push, dry_run).await
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
        Some(r) => router::route(&auth, &event, &service, &sandbox, &r).await,
        None => println!("Rule not specified"),
    }
}

pub async fn freeze(auth: Auth, sandbox: Option<String>) {
    let topology = composer::compose(&u::pwd(), true);
    let sandbox = resolver::maybe_sandbox(sandbox);
    let topology = resolver::render(&auth, &sandbox, &topology).await;
    deployer::freeze(&auth, &topology).await;
    let msg = format!("*{}*::{} is frozen", &auth.name, sandbox);
    notifier::notify(&topology.namespace, &msg).await;
}

pub async fn unfreeze(auth: Auth, sandbox: Option<String>) {
    let topology = composer::compose(&u::pwd(), true);
    let sandbox = resolver::maybe_sandbox(sandbox);
    let topology = resolver::render(&auth, &sandbox, &topology).await;
    deployer::unfreeze(&auth, &topology).await;
    let msg = format!("*{}*::{} is unfrozen", &auth.name, sandbox);
    notifier::notify(&topology.namespace, &msg).await
}

pub async fn upgrade(version: Option<String>) {
    u::self_upgrade("tc", version).await
}

pub async fn show_config() {
    let config = Config::new();
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

pub struct SnapshotOpts {
    pub save: bool,
    pub format: Option<String>,
    pub target_env: Option<String>,
    pub target_sandbox: Option<String>,
    pub gen_changelog: bool,
    pub gen_sub_versions: bool,
}

pub async fn snapshot(profile: Option<String>, sandbox: Option<String>, opts: SnapshotOpts) {
    let SnapshotOpts {
        save,
        format,
        gen_changelog,
        gen_sub_versions,
        target_env,
        target_sandbox,
    } = opts;

    let dir = u::root();
    let format = u::maybe_string(format, "table");
    let sandbox = u::maybe_string(sandbox, "stable");

    let format = if gen_changelog || gen_sub_versions {
        "json"
    } else {
        &format
    };

    match profile {
        Some(ref p) => {
            let profiles: Vec<String> = p.split(",").map(|v| v.to_string()).collect();
            if profiles.len() > 1 {
                snapshotter::snapshot_profiles(&dir, &sandbox, profiles).await;
            } else {
                let auth = init(profile.clone(), None).await;

                let records = snapshotter::snapshot(&auth, &dir, &sandbox, gen_changelog).await;

                if save {
                    let records_str = serde_json::to_string_pretty(&records).unwrap();
                    snapshotter::save(&auth, &records_str, &p, &sandbox).await
                }
                snapshotter::pretty_print(&records, &format, target_env, target_sandbox);
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
                    let version = tagger::find_version_history(&namespace, &s).await;
                    if let Some(v) = version {
                        println!("{},{},{}", &s, &namespace, &v);
                    }
                }
            }
        }
        None => tagger::changelog(&namespace, between, verbose),
    }
}

pub async fn prune(auth: &Auth, sandbox: Option<String>, filter: Option<String>, dry_run: bool) {
    match sandbox {
        Some(sbox) => {
            if dry_run {
                deployer::list_all(auth, &sbox, "default").await;
            } else {
                deployer::guard::prevent_stable_updates(&sbox);
                deployer::prune(auth, &sbox, filter).await;
            }
        }
        None => println!("Please specify sandbox"),
    }
}

pub async fn list(auth: &Auth, sandbox: Option<String>, entity: Option<String>) {
    let topology = composer::compose(&u::pwd(), true);
    let sandbox = resolver::maybe_sandbox(sandbox);
    let topology = resolver::render(&auth, &sandbox, &topology).await;
    deployer::try_list(auth, &topology, &entity).await;
}

pub async fn list_all(auth: &Auth, sandbox: Option<String>, format: Option<String>) {
    let sandbox = resolver::maybe_sandbox(sandbox);
    let format = u::maybe_string(format, "default");
    deployer::list_all(auth, &sandbox, &format).await;
}

pub fn scaffold(kind: Option<String>) {
    let kind = u::maybe_string(kind, "function");
    scaffolder::scaffold(&kind)
}

pub async fn emulate(
    auth: &Auth,
    sandbox: Option<String>,
    maybe_entity: Option<String>,
    shell: bool,
) {
    let sandbox = u::maybe_string(sandbox, "stable");
    let topology = composer::compose(&u::pwd(), false);
    let rt = resolver::try_resolve(&auth, &sandbox, &topology, &maybe_entity, false, true).await;
    let entity_component = u::maybe_string(maybe_entity, "function");
    emulator::emulate(auth, &rt, &entity_component, shell).await;
}
