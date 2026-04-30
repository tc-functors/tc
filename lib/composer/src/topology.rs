pub use crate::aws::{
    channel::Channel,
    event::Event,
    flow::Flow,
    function::{
        Function,
        layer,
        layer::Layer,
    },
    mutation::{
        Mutation,
        Resolver,
    },
    page::Page,
    pool::Pool,
    queue::Queue,
    role::Role,
    route::Route,
    schedule::Schedule,
    transducer::Transducer,
};
use crate::{
    aws::{
        channel,
        event,
        mutation,
        page,
        pool,
        schedule,
        template,
    },
    sequence,
    tag,
};
use compiler::{
    Entity,
    spec::{
        TestSpec,
        TopologyKind,
        TopologySpec,
    },
};
use configurator::Config;
use kit as u;
use kit::*;
pub use sequence::Connector;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::Path,
};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Topology {
    pub namespace: String,
    pub env: String,
    pub fqn: String,
    pub concurrent: bool,
    pub kind: TopologyKind,
    pub infra: String,
    pub dir: String,
    pub sandbox: String,
    pub hyphenated_names: bool,
    pub version: String,
    pub nodes: HashMap<String, Topology>,
    pub events: HashMap<String, Event>,
    pub routes: HashMap<String, Route>,
    pub functions: HashMap<String, Function>,
    pub mutations: HashMap<String, Mutation>,
    pub schedules: HashMap<String, Schedule>,
    pub queues: HashMap<String, Queue>,
    pub channels: HashMap<String, Channel>,
    pub pools: HashMap<String, Pool>,
    pub pages: HashMap<String, Page>,
    pub tags: HashMap<String, String>,
    pub flow: Option<Flow>,
    pub config: Config,
    pub roles: HashMap<String, Role>,
    pub base_roles: HashMap<String, Role>,
    pub tests: HashMap<String, TestSpec>,
    pub transducer: Option<Transducer>,
    pub sequences: HashMap<String, Vec<Connector>>,
}

fn relative_root_path(dir: &str) -> (String, String) {
    let root = u::split_first(&dir, "/topologies/");
    let next = u::second(&dir, "/topologies/");
    (root, next)
}

fn as_infra_dir(given_infra_dir: Option<String>, topology_dir: &str) -> String {
    match given_infra_dir {
        Some(d) => d,
        None => {
            let (root, next) = relative_root_path(topology_dir);
            let s = next.replace("_", "-");
            format!("{root}/infrastructure/tc/{s}")
        }
    }
}

pub fn is_topology_dir(dir: &str) -> bool {
    let topology_file = format!("{}/topology.yml", dir);
    Path::new(&topology_file).exists()
}

fn parent_topology_file(dir: &str) -> Option<String> {
    let paths = vec![
        u::absolutize(dir, "../topology.yml"),
        u::absolutize(dir, "../../topology.yml"),
        u::absolutize(dir, "../../../topology.yml"),
        u::absolutize(dir, "../../../../topology.yml"),
        s!("../topology.yml"),
        s!("../../topology.yml"),
        s!("../../../topology.yml"),
        s!("../../../../topology.yml"),
    ];
    u::any_path(paths)
}

pub fn is_root_topology(spec_file: &str) -> bool {
    let spec = TopologySpec::new(spec_file);
    if let Some(given_root_dirs) = &spec.nodes.dirs {
        !given_root_dirs.is_empty()
    } else {
        if let Some(root) = spec.root {
            root
        } else {
            false
        }
    }
}

pub fn is_relative_topology_dir(dir: &str) -> bool {
    let topology_file = parent_topology_file(dir);
    match topology_file {
        Some(file) => {
            if is_root_topology(&file) {
                false
            } else {
                Path::new(&file).exists()
            }
        }
        None => false,
    }
}

// functions

fn is_standalone_function_dir(dir: &str) -> bool {
    let function_file = &compiler::spec::function::find_fspec_file(dir);
    let function_file_json = "function.json";
    let topology_file = "topology.yml";
    let parent_file = match parent_topology_file(dir) {
        Some(file) => file,
        None => u::empty(),
    };
    if is_root_topology(&parent_file) {
        return true;
    } else {
        (u::file_exists(function_file) || u::file_exists(function_file_json))
            && !u::file_exists(topology_file)
            && !u::file_exists(&parent_file)
            || u::file_exists("handler.rb")
            || u::file_exists("handler.py")
            || u::file_exists("main.go")
            || u::file_exists("Cargo.toml")
            || u::file_exists("handler.janet")
            || u::file_exists("handler.clj")
            || u::file_exists("handler.js")
            || u::file_exists("main.janet")
    }
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    u::file_exists(function_file) && u::file_exists(topology_file)
}

fn is_shared(uri: Option<String>) -> bool {
    match uri {
        Some(p) => p.starts_with("."),
        None => false,
    }
}

fn abs_shared_dir(root_dir: &str, uri: Option<String>) -> String {
    match uri {
        Some(p) => u::absolute_dir(&root_dir, &p),
        None => panic!("Shared uri not specified"),
    }
}

fn intern_functions(
    root_namespace: &str,
    infra_dir: &str,
    spec: &TopologySpec,
) -> HashMap<String, Function> {
    let inline_fns = match &spec.functions {
        Some(f) => f,
        None => &HashMap::new(),
    };

    let mut fns: HashMap<String, Function> = HashMap::new();
    let root_dir = &spec.dir.clone().unwrap();

    for (name, f) in inline_fns {
        if is_shared(f.uri.clone()) {
            let abs_dir = abs_shared_dir(root_dir, f.uri.clone());
            let namespace = match &f.fqn {
                Some(_) => &spec.name,
                None => root_namespace,
            };

            let mut function = Function::new(&abs_dir, infra_dir, &namespace, spec.fmt());
            function.shared = true;
            fns.insert(s!(name), function);
        } else {
            let dir = format!("{}/{}", root_dir, name);
            let namespace = &spec.name;
            let fspec = f.intern(namespace, &dir, infra_dir, &name);
            let function = Function::from_spec(&fspec, namespace, &dir, infra_dir);
            fns.insert(s!(name), function);
        }
    }
    fns
}

fn is_inferred_dir(dir: &str) -> bool {
    u::path_exists(dir, "handler.rb")
        || u::path_exists(dir, "handler.py")
        || u::path_exists(dir, "main.go")
        || u::path_exists(dir, "Cargo.toml")
        || u::path_exists(dir, "handler.janet")
        || u::path_exists(dir, "handler.clj")
        || u::path_exists(dir, "handler.js")
        || u::path_exists(dir, "main.janet")
}

fn function_dirs(dir: &str) -> Vec<String> {
    let known_roots = vec!["resolvers", "functions", "transformers"];
    let mut xs: Vec<String> = vec![];
    let dirs = u::list_dirs(dir);
    for root in known_roots {
        let mut xm = u::list_dirs(root);
        xs.append(&mut xm)
    }
    for d in dirs {
        if path_exists(&d, "function.yml")
            || path_exists(&d, "function.json")
            || is_inferred_dir(&d)
        {
            xs.push(d.to_string())
        }
    }
    xs
}

fn ignore_function(dir: &str, root_dir: &str) -> bool {
    let ignore_file = u::path_of(root_dir, ".tcignore");
    if dir.contains(".circleci")
        || dir.contains(".git")
        || dir.contains(".vendor")
        || dir.contains(".venv")
        || dir.contains(".env")
        || dir.contains("node_modules")
        || dir.ends_with("states")
        || dir.ends_with("topology")
        || dir.ends_with("roles")
        || dir.ends_with("extensions")
    {
        return true;
    }
    if u::file_exists(&ignore_file) {
        let globs = u::readlines(&ignore_file);
        for g in globs {
            if u::is_dir(&g) && dir.ends_with(&g) {
                return true;
            } else {
                continue;
            }
        }
        return false;
    } else {
        false
    }
}

fn discover_functions_sequential(
    dir: &str,
    infra_dir: &str,
    spec: &TopologySpec,
) -> HashMap<String, Function> {
    function_dirs(dir)
        .into_iter()
        .filter(|d| u::is_dir(d) && !ignore_function(d, dir))
        .map(|d| {
            tracing::debug!("function {}", d);
            let f = Function::new(&d, infra_dir, &spec.name, spec.fmt());
            (f.name.clone(), f)
        })
        .collect()
}

fn discover_functions(
    dir: &str,
    infra_dir: &str,
    spec: &TopologySpec,
) -> HashMap<String, Function> {
    let dirs: Vec<String> = function_dirs(dir)
        .into_iter()
        .filter(|d| u::is_dir(d) && !ignore_function(d, dir))
        .collect();
    if dirs.is_empty() {
        return HashMap::new();
    }

    let name = spec.name.clone();
    let fmt = spec.fmt();
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(dirs.len());
    let chunk_size = dirs.len().div_ceil(threads);

    std::thread::scope(|s| {
        dirs.chunks(chunk_size)
            .map(|chunk| {
                s.spawn(|| {
                    chunk
                        .iter()
                        .map(|d| {
                            tracing::debug!("function {}", d);
                            let f = Function::new(d, infra_dir, &name, &fmt);
                            (f.name.clone(), f)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    })
}

fn current_function(dir: &str, infra_dir: &str, spec: &TopologySpec) -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    if u::is_dir(dir) && !dir.starts_with(".") {
        let function = Function::new(dir, infra_dir, &spec.name, spec.fmt());
        functions.insert(function.name.to_string(), function);
    }
    functions
}

// nodes

fn should_ignore_node(
    root_dir: &str,
    ignore_nodes: Option<Vec<String>>,
    topology_dir: &str,
) -> bool {
    let ignore_file = u::path_of(root_dir, ".tcignore");
    if u::file_exists(&ignore_file) {
        let globs = u::readlines(&ignore_file);
        for g in globs {
            let gdir = format!("{}/{}", root_dir, &g);
            if topology_dir.ends_with(&g) || topology_dir.contains(&gdir) {
                return true;
            } else {
                continue;
            }
        }
        return false;
    } else {
        if let Some(ignodes) = ignore_nodes {
            for node in ignodes {
                let abs_path = format!("{root_dir}/{node}");
                if &abs_path == topology_dir {
                    return true;
                }
                if topology_dir.starts_with(&abs_path) {
                    return true;
                }
                return false;
            }
        }
        return false;
    }
}

fn discover_leaf_nodes(
    root_ns: &str,
    root_dir: &str,
    dir: &str,
    s: &TopologySpec,
) -> HashMap<String, Topology> {
    let ignore_nodes = &s.nodes.ignore;

    let mut nodes: HashMap<String, Topology> = HashMap::new();
    if is_topology_dir(dir) {
        if !should_ignore_node(root_dir, ignore_nodes.clone(), dir) {
            let f = format!("{}/topology.yml", dir);
            let spec = TopologySpec::new(&f);
            let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
            let mut functions = discover_functions(dir, &infra_dir, &spec);
            let interned = intern_functions(root_ns, &infra_dir, &spec);
            functions.extend(interned);
            let node = make(root_dir, dir, &spec, functions, HashMap::new());
            nodes.insert(spec.name.to_string(), node);
        }
    }
    nodes
}

// builders

fn make_nodes(root_dir: &str, spec: &TopologySpec) -> HashMap<String, Topology> {
    let ignore_nodes = &spec.nodes.ignore;
    let candidates: Vec<String> = WalkDir::new(root_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_string_lossy().to_string())
        .filter(|p| is_topology_dir(p) && root_dir != p)
        .filter(|p| !should_ignore_node(root_dir, ignore_nodes.clone(), p))
        .collect();
    if candidates.is_empty() {
        return HashMap::new();
    }

    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(candidates.len());
    let chunk_size = candidates.len().div_ceil(threads);

    std::thread::scope(|s| {
        candidates
            .chunks(chunk_size)
            .map(|chunk| {
                s.spawn(|| {
                    chunk
                        .iter()
                        .map(|p| {
                            let f = format!("{}/topology.yml", &p);
                            let node_spec = TopologySpec::new(&f);
                            tracing::debug!("node {}", &node_spec.name);
                            let infra_dir = as_infra_dir(node_spec.infra.to_owned(), p);
                            let mut functions =
                                discover_functions_sequential(p, &infra_dir, &node_spec);
                            let interned = intern_functions(p, &infra_dir, &node_spec);
                            functions.extend(interned);
                            let leaf_nodes =
                                discover_leaf_nodes(&node_spec.name, root_dir, p, &node_spec);
                            let node = make(root_dir, p, &node_spec, functions, leaf_nodes);
                            (node_spec.name.to_string(), node)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    })
}

fn make_events(
    namespace: &str,
    spec: &TopologySpec,
    fqn: &str,
    config: &Config,
    fns: &HashMap<String, Function>,
    resolvers: &HashMap<String, Resolver>,
) -> HashMap<String, Event> {
    let events = &spec.events;
    let mut h: HashMap<String, Event> = HashMap::new();
    if let Some(evs) = events {
        for (name, espec) in evs {
            tracing::debug!("event {}", &name);
            let targets = event::make_targets(namespace, &name, &espec, fqn, fns, resolvers);
            let skip = espec.doc_only;
            let ev = Event::new(&name, &espec, targets, config, skip);
            h.insert(name.to_string(), ev);
        }
    }
    h
}

fn make_routes(
    spec: &TopologySpec,
    fqn: &str,
    fns: &HashMap<String, Function>,
    events: &HashMap<String, Event>,
    queues: &HashMap<String, Queue>,
    infra_dir: &str,
) -> HashMap<String, Route> {
    let routes = &spec.routes;
    match routes {
        Some(xs) => {
            let mut h: HashMap<String, Route> = HashMap::new();
            for (name, rspec) in xs {
                tracing::debug!("route {}", &name);
                let skip = rspec.doc_only;
                let route = Route::new(
                    fqn, &name, spec, rspec, fns, events, queues, infra_dir, skip,
                );
                h.insert(name.to_string(), route);
            }
            h
        }
        None => HashMap::new(),
    }
}

fn make_queues(spec: &TopologySpec, _config: &Config) -> HashMap<String, Queue> {
    let mut h: HashMap<String, Queue> = HashMap::new();
    if let Some(queues) = &spec.queues {
        tracing::debug!("Compiling queues");
        for (name, qspec) in queues {
            h.insert(name.to_string(), Queue::new(&name, qspec));
        }
    }
    h
}

fn make_mutations(spec: &TopologySpec, _config: &Config) -> HashMap<String, Mutation> {
    let mutations = mutation::make(&spec.name, spec.mutations.to_owned());
    let mut h: HashMap<String, Mutation> = HashMap::new();
    if let Some(ref m) = mutations {
        tracing::debug!("Compiling mutations");
        h.insert(s!("default"), m.clone());
    }
    h
}

fn make_channels(spec: &TopologySpec, _config: &Config) -> HashMap<String, Channel> {
    match &spec.channels {
        Some(c) => channel::make(&spec.name, c.clone()),
        None => HashMap::new(),
    }
}

fn make_pools(spec: &TopologySpec, config: &Config) -> HashMap<String, Pool> {
    let pools = match &spec.pools {
        Some(p) => p.clone(),
        None => vec![],
    };
    match &spec.triggers {
        Some(c) => pool::make(pools, c.clone(), config),
        None => HashMap::new(),
    }
}

fn make_roles(
    functions: &HashMap<String, Function>,
    _mutations: &usize,
    _routes: &usize,
    _events: &usize,
    states: &Option<Flow>,
) -> HashMap<String, Role> {
    let mut h: HashMap<String, Role> = HashMap::new();
    for (_, f) in functions {
        if &f.runtime.role.kind.to_str() != "provided" {
            let role = f.runtime.role.clone();
            h.insert(role.name.clone(), role);
        }
    }

    if let Some(f) = states {
        let role = &f.role;
        h.insert(role.name.clone(), role.clone());
    }
    h
}

fn make_base_roles() -> HashMap<String, Role> {
    let mut h: HashMap<String, Role> = HashMap::new();

    h.insert("mutation".to_string(), Role::default(Entity::Mutation));
    h.insert("route".to_string(), Role::default(Entity::Route));
    h.insert("event".to_string(), Role::default(Entity::Event));
    h.insert("state".to_string(), Role::default(Entity::State));
    h.insert("function".to_string(), Role::default(Entity::Function));
    h
}


fn make_test(
    t: Option<HashMap<String, TestSpec>>,
    fns: &HashMap<String, Function>,
) -> HashMap<String, TestSpec> {
    let mut tspecs = match t {
        Some(spec) => spec,
        None => HashMap::new(),
    };
    for (fname, f) in fns {
        for (name, mut tspec) in f.test.clone() {
            tspec.entity = Some(format!("function/{}", &fname));
            tspecs.insert(name.to_string(), tspec.clone());
        }
    }
    tspecs
}

fn find_kind(
    given_kind: &Option<TopologyKind>,
    flow: &Option<Flow>,
    functions: &HashMap<String, Function>,
    mutations: &HashMap<String, Mutation>,
    routes: &HashMap<String, Route>,
) -> TopologyKind {
    match given_kind {
        Some(k) => k.clone(),
        None => match flow {
            Some(_) => TopologyKind::StepFunction,
            None => {
                if !mutations.is_empty() {
                    return TopologyKind::Graphql;
                } else if !routes.is_empty() {
                    return TopologyKind::Routed;
                } else if !functions.is_empty() {
                    return TopologyKind::Function;
                } else {
                    return TopologyKind::Evented;
                }
            }
        },
    }
}

fn is_concurrent(maybe_concurrent: &Option<bool>) -> bool {
    match maybe_concurrent {
        Some(b) => *b,
        None => false
    }
}

fn make(
    _root_dir: &str,
    dir: &str,
    spec: &TopologySpec,
    functions: HashMap<String, Function>,
    nodes: HashMap<String, Topology>,
) -> Topology {
    let config = Config::new();

    let mut functions = functions;
    let namespace = spec.name.to_owned();
    let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
    tracing::debug!(
        "node-infra-dir {:?}, {} {}",
        &spec.infra,
        &spec.name,
        &infra_dir
    );
    let interned = intern_functions(&namespace, &infra_dir, &spec);
    functions.extend(interned);

    let version = u::current_semver(&namespace);
    let fqn = template::topology_fqn(&namespace, spec.hyphenated_names);
    let flow = Flow::new(dir, &infra_dir, &fqn, &spec);
    let mutations = make_mutations(&spec, &config);

    let resolvers = match &mutations.get("default") {
        Some(m) => m.resolvers.clone(),
        None => HashMap::new(),
    };
    let events = make_events(&namespace, &spec, &fqn, &config, &functions, &resolvers);
    let queues = make_queues(&spec, &config);
    let routes = make_routes(&spec, &fqn, &functions, &events, &queues, &infra_dir);
    let channels = make_channels(&spec, &config);

    let maybe_transducer = Transducer::new(&namespace, &functions, &events, &mutations, &channels);

    Topology {
        namespace: namespace.clone(),
        fqn: fqn.clone(),
        concurrent: is_concurrent(&spec.concurrent),
        env: template::profile(),
        kind: find_kind(&spec.kind, &flow, &functions, &mutations, &routes),
        version: version,
        infra: u::gdir(&infra_dir),
        sandbox: template::sandbox(),
        dir: dir.to_string(),
        hyphenated_names: spec.hyphenated_names.to_owned(),
        nodes: nodes,
        roles: make_roles(
            &functions,
            &resolvers.len(),
            &routes.len(),
            &events.len(),
            &flow,
        ),
        base_roles: make_base_roles(),
        events: events,
        routes: routes,
        tests: make_test(spec.tests.clone(), &functions),
        functions: functions,
        schedules: schedule::make_all(&namespace, &infra_dir),
        queues: queues,
        mutations: mutations,
        channels: channels,
        pools: make_pools(&spec, &config),
        tags: tag::make(&spec.name, &infra_dir),
        pages: page::make_all(&spec, &infra_dir, &config),
        flow: flow,
        config: Config::new(),
        transducer: maybe_transducer,
        sequences: sequence::make_all(&spec.sequences),
    }
}

fn make_relative(dir: &str) -> Topology {
    let f = match parent_topology_file(dir) {
        Some(file) => file,
        None => format!("../topology.yml"),
    };

    let spec = TopologySpec::new(&f);
    let namespace = &spec.name;
    let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
    let function = Function::new(dir, &infra_dir, namespace, &spec.fmt());
    let functions = Function::to_map(function);
    let nodes = HashMap::new();
    make(dir, dir, &spec, functions, nodes)
}

fn make_standalone(dir: &str) -> Topology {
    let function = Function::new(dir, dir, "", "");
    let functions = Function::to_map(function.clone());
    let namespace = function.name.to_owned();

    Topology {
        namespace: namespace.clone(),
        env: template::profile(),
        fqn: template::topology_fqn(&namespace, false),
        kind: TopologyKind::Function,
        concurrent: false,
        version: u::current_semver(&namespace),
        sandbox: template::sandbox(),
        infra: u::empty(),
        dir: s!(dir),
        hyphenated_names: false,
        events: HashMap::new(),
        routes: HashMap::new(),
        flow: None,
        pools: HashMap::new(),
        roles: make_roles(&functions, &0, &0, &0, &None),
        base_roles: make_base_roles(),
        functions: functions,
        nodes: HashMap::new(),
        mutations: HashMap::new(),
        queues: HashMap::new(),
        channels: HashMap::new(),
        tags: tag::make(&namespace, ""),
        schedules: HashMap::new(),
        pages: HashMap::new(),
        tests: HashMap::new(),
        config: Config::new(),
        transducer: None,
        sequences: HashMap::new(),
    }
}

pub fn is_compilable(dir: &str) -> bool {
    is_standalone_function_dir(dir) || is_relative_topology_dir(dir) || is_topology_dir(dir)
}

// Shared-function deduplication: single-owner promotion to root.
//
// `intern_functions` materialises a fresh `Function` into the importing
// node's `functions` map for every entry whose `uri:` is a relative path
// (see `is_shared`). Without dedup, when N nested topologies import the
// same shared function, the resulting recursive `Topology` contains N
// copies — one per importer — and `tc create` ends up running N full
// Docker builds + N redundant `lambda:CreateFunction`/`UpdateFunctionCode`
// calls against the same `fqn`.
//
// The fix is a post-pass over the recursively-built root topology that
// drains every `Function` with `shared = true` out of each descendant's
// `functions` map and inserts it into `root.functions`, deduped by
// HashMap key. After this pass:
//
//   - `root.functions` contains every shared function exactly once
//   - each descendant's `functions` map contains only its own (non-shared)
//     entries
//   - downstream callers (`deployer::create`, `builder::build_recursive`,
//     `function::resolve`, etc.) deploy/build/resolve each shared
//     function exactly once.
//
// The promotion runs only when `Topology::new` is constructing a
// recursive root topology. Non-recursive callers (single-leaf modes
// like `tc compose` on one node, `current_function`, `is_compilable`)
// don't have descendants to dedupe against.

/// Recursively drains all `shared` functions out of `node` and its
/// descendants into `target`. Conflicts (same map key) resolve to
/// `target`'s existing entry; identical-fqn collisions are silent,
/// fqn-mismatched collisions emit `tracing::debug!` so latent name
/// collisions don't go unnoticed. Each topology's own (non-shared)
/// functions are preserved in place.
fn drain_shared_into(target: &mut HashMap<String, Function>, node: &mut Topology) {
    for child in node.nodes.values_mut() {
        drain_shared_into(target, child);
    }
    let drained = std::mem::take(&mut node.functions);
    let mut owned: HashMap<String, Function> = HashMap::with_capacity(drained.len());
    for (name, f) in drained {
        if f.shared {
            match target.entry(name.clone()) {
                std::collections::hash_map::Entry::Occupied(existing) => {
                    if existing.get().fqn != f.fqn {
                        tracing::debug!(
                            "shared-function key collision: {} ({} vs {}) — keeping existing",
                            name,
                            existing.get().fqn,
                            f.fqn
                        );
                    }
                }
                std::collections::hash_map::Entry::Vacant(slot) => {
                    slot.insert(f);
                }
            }
        } else {
            owned.insert(name, f);
        }
    }
    node.functions = owned;
}

/// Counts every `shared = true` function in `t` and all its descendants.
/// Used purely for the post-promotion log line that reports how many
/// duplicate imports were eliminated.
fn count_shared_recursive(t: &Topology) -> usize {
    let here = t.functions.values().filter(|f| f.shared).count();
    let descendants: usize = t.nodes.values().map(count_shared_recursive).sum();
    here + descendants
}

/// Recompute every topology's `roles` map from its post-promotion
/// `functions` map and `flow`. Necessary because `make()` calls
/// `make_roles` *before* `promote_shared_to_root` runs, leaving the
/// `roles` maps stale relative to the new `functions` shape:
///
/// - Root's `roles` is missing the shared functions' roles (root
///   didn't own those functions when `make_roles` ran).
/// - Each descendant's `roles` still contains the shared functions'
///   roles (the descendant owned them at make-time but no longer
///   does after promotion).
///
/// Without this pass, a fresh `tc create` deploys root first
/// (`deployer::create` calls `role::create_or_update` then
/// `function::create`); since root's IAM roles for the shared
/// functions don't exist yet, AWS rejects the Lambda creation calls
/// (the Lambdas reference role ARNs that haven't been provisioned).
///
/// Recomputation is fully equivalent to the original computation:
/// `make_roles` is a pure function of `(functions, flow)`, both of
/// which are stable through promotion (only the *placement* of
/// functions changes, never their `runtime.role` payload).
fn recompute_roles_recursive(t: &mut Topology) {
    for child in t.nodes.values_mut() {
        recompute_roles_recursive(child);
    }
    t.roles = make_roles(&t.functions, &0, &0, &0, &t.flow);
}

/// Single-owner invariant: after this call, every `Function` with
/// `shared = true` lives in `root.functions`, every descendant's
/// `functions` map contains only that descendant's own (non-shared)
/// functions, and every topology's `roles` map matches its current
/// `functions` map.
///
/// Conflict semantics (`or_insert` via `Entry::Vacant`):
///
/// - Two siblings declare the same shared function → first one wins,
///   others dropped. Equivalent because they all reference the same
///   source dir.
/// - Root *and* a node declare the same shared function → root's entry
///   wins. Same source dir, equivalent.
/// - Two functions with the same map key but different `fqn`s → first
///   one wins, others dropped. Doesn't happen in practice (keys are
///   local function names and shared functions have a single canonical
///   `function.yml`); emits `tracing::debug!` if it ever does.
pub(crate) fn promote_shared_to_root(root: &mut Topology) {
    let nodes_count = root.nodes.len();
    let before: usize = root
        .nodes
        .values()
        .map(count_shared_recursive)
        .sum();
    let mut promoted: HashMap<String, Function> = HashMap::new();
    for child in root.nodes.values_mut() {
        drain_shared_into(&mut promoted, child);
    }
    let promoted_count = promoted.len();
    for (name, f) in promoted {
        root.functions.entry(name).or_insert(f);
    }
    // `make_roles` ran inside each `make()` call before this pass, so
    // every topology's `roles` map is now stale w.r.t. its current
    // `functions` map. Restore the invariant before downstream
    // consumers (deployer, builder, etc.) read the topology.
    recompute_roles_recursive(root);
    if before > 0 {
        tracing::info!(
            "Promoted {} unique shared function(s) from {} node(s) to root (eliminated {} duplicate import(s))",
            promoted_count,
            nodes_count,
            before.saturating_sub(promoted_count),
        );
    }
}

impl Topology {
    pub fn new(dir: &str, recursive: bool, skip_functions: bool) -> Topology {
        if is_singular_function_dir() {
            let f = format!("{}/topology.yml", dir);
            let spec = TopologySpec::new(&f);
            let infra_dir = as_infra_dir(spec.infra.to_owned(), &spec.name);
            let functions = current_function(dir, &infra_dir, &spec);
            make(dir, dir, &spec, functions, HashMap::new())
        } else if is_topology_dir(dir) {
            let f = format!("{}/topology.yml", dir);
            let spec = TopologySpec::new(&f);
            let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
            tracing::debug!("Infra dir: {}  {}", &spec.name, &infra_dir);

            let nodes = if recursive {
                tracing::debug!("Recursive {}", dir);
                make_nodes(dir, &spec)
            } else {
                HashMap::new()
            };
            let mut topology = if skip_functions {
                tracing::debug!("Skipping functions {}", dir);
                make(dir, dir, &spec, HashMap::new(), nodes)
            } else {
                tracing::debug!("Discovering functions {}", dir);
                let functions = discover_functions(dir, &infra_dir, &spec);
                make(dir, dir, &spec, functions, nodes)
            };
            if recursive {
                promote_shared_to_root(&mut topology);
            }
            topology
        } else if is_relative_topology_dir(dir) {
            make_relative(dir)
        } else if is_standalone_function_dir(dir) {
            make_standalone(dir)
        } else {
            println!("{}", dir);
            std::panic::set_hook(Box::new(|_| {
                println!("No topology.yml or function.json found. Inference failed");
            }));
            panic!("Don't know what to do");
        }
    }

    pub fn functions(&self) -> HashMap<String, Function> {
        let mut fns: HashMap<String, Function> = self.clone().functions;
        for (_, node) in &self.nodes {
            fns.extend(node.clone().functions);
        }
        fns.clone()
    }

    pub fn current_function(&self, dir: &str) -> Option<Function> {
        let fns: HashMap<String, Function> = self.clone().functions;
        for (_, f) in fns {
            if f.dir == dir {
                return Some(f);
            }
        }
        None
    }

    pub fn layers(&self) -> Vec<Layer> {
        let fns = self.functions();
        layer::find(fns)
    }

    pub fn to_str(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn from_json(v: Value) -> Topology {
        let t: Topology = serde_json::from_value(v).unwrap();
        t
    }

    pub fn to_bincode(&self) {
        let byea: Vec<u8> = bincode::serialize(self).unwrap();
        let path = format!("{}-{}.tc", self.fqn, self.version);
        kit::write_bytes(&path, byea);
    }

    pub fn read_bincode(path: &str) -> Topology {
        let data = kit::read_bytes(path);
        let t: Topology = bincode::deserialize(&data).unwrap();
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Minimum function.yml that satisfies `FunctionSpec::new` →
    /// `Runtime::new` → `make_lambda` without panicking. `fqn:` is set
    /// so all importers compute the same final fqn (`shared_foo_<sandbox>`)
    /// regardless of which node's namespace was passed in to `make_fqn`,
    /// which mirrors how shared functions are written in real codebases.
    fn write_shared_function(dir: &std::path::Path, name: &str, fqn: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join("function.yml"),
            format!(
                "name: {name}\n\
                 fqn: {fqn}\n\
                 runtime:\n  \
                   lang: python3.10\n  \
                   handler: handler.handler\n  \
                   package_type: zip\n  \
                   layers: []\n\
                 build:\n  \
                   kind: Code\n  \
                   command: echo build\n"
            ),
        )
        .unwrap();
        // Satisfy `is_inferred_dir`. Empty file is fine — we never invoke
        // the build pipeline, just discover/intern.
        fs::write(dir.join("handler.py"), "").unwrap();
    }

    fn write_topology_yml(dir: &std::path::Path, contents: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("topology.yml"), contents).unwrap();
    }

    /// Reproduces the consumer-extraction shape that triggered the
    /// shared-function rebuild bug: a parent topology with multiple
    /// nested topologies, each declaring the same shared function via
    /// a relative `uri:` in its `functions:` block. After recursive
    /// composition, the shared function must appear exactly once at
    /// root and zero times in any descendant.
    ///
    /// Also covers, in the same fixture:
    /// - A second shared function imported by a subset of nodes
    ///   (verifies the post-pass dedupes per-key, not per-node).
    /// - A descendant with its own non-shared function declared inline
    ///   (verifies that owned functions are preserved in place).
    /// - JSON round-trip of a promoted Function with the `shared` field
    ///   stripped (regression guard for the `#[serde(default)]`
    ///   attribute on `Function.shared`).
    #[test]
    fn compose_recursive_dedups_shared_functions_to_root() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();

        // Parent topology — no inline functions, no flow.
        write_topology_yml(
            root,
            "name: shared-dedup-parent\nkind: step-function\n",
        );

        // Two shared function source dirs, both imported by descendants.
        write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");
        write_shared_function(&root.join("shared/bar"), "bar", "shared_bar");

        // Three child topologies, all importing `foo`. Only `c` also
        // imports `bar`. `c` additionally declares its own non-shared
        // function `local`, to confirm the post-pass leaves owned
        // entries in place.
        write_topology_yml(
            &root.join("a"),
            "name: child-a\nkind: step-function\nfunctions:\n  foo:\n    uri: ../shared/foo\n",
        );
        write_topology_yml(
            &root.join("b"),
            "name: child-b\nkind: step-function\nfunctions:\n  foo:\n    uri: ../shared/foo\n",
        );
        write_topology_yml(
            &root.join("c"),
            "name: child-c\n\
             kind: step-function\n\
             functions:\n  \
               foo:\n    uri: ../shared/foo\n  \
               bar:\n    uri: ../shared/bar\n",
        );
        // c's own non-shared function lives at c/local/. Discovered, not interned.
        write_shared_function(&root.join("c/local"), "local", "child_c_local");

        let topology = Topology::new(root.to_str().unwrap(), true, false);

        // Root owns both shared functions exactly once each.
        assert!(
            topology.functions.contains_key("foo"),
            "shared function `foo` must be promoted to root.functions; root has {:?}",
            topology.functions.keys().collect::<Vec<_>>()
        );
        assert!(
            topology.functions.contains_key("bar"),
            "shared function `bar` must be promoted to root.functions; root has {:?}",
            topology.functions.keys().collect::<Vec<_>>()
        );
        assert!(
            topology
                .functions
                .get("foo")
                .map(|f| f.shared)
                .unwrap_or(false),
            "promoted `foo` retains shared = true"
        );
        assert!(
            topology
                .functions
                .get("bar")
                .map(|f| f.shared)
                .unwrap_or(false),
            "promoted `bar` retains shared = true"
        );

        // No descendant retains either shared function (search the full
        // descendant tree, not just direct children, since
        // `discover_leaf_nodes` mirrors each node into its own
        // `node.nodes` map and a future refactor could turn this into
        // genuine multi-level nesting).
        fn collect_offenders(
            t: &Topology,
            path: String,
            keys: &[&'static str],
            out: &mut Vec<(String, &'static str)>,
        ) {
            for k in keys {
                if t.functions.contains_key(*k) {
                    out.push((path.clone(), *k));
                }
            }
            for (name, child) in &t.nodes {
                collect_offenders(child, format!("{path}/{name}"), keys, out);
            }
        }

        let mut offenders: Vec<(String, &'static str)> = Vec::new();
        for (name, child) in &topology.nodes {
            collect_offenders(child, name.clone(), &["foo", "bar"], &mut offenders);
        }
        assert!(
            offenders.is_empty(),
            "no descendant should retain a shared function; found: {:?}",
            offenders
        );

        // Three nested topologies must be present at root level
        // (`make_nodes` flattens the WalkDir result, so all three are
        // direct children of root regardless of fs depth).
        let child_namespaces: std::collections::BTreeSet<&String> =
            topology.nodes.keys().collect();
        assert!(
            child_namespaces.contains(&s!("child-a")),
            "child-a missing; root.nodes = {:?}",
            child_namespaces
        );
        assert!(
            child_namespaces.contains(&s!("child-b")),
            "child-b missing; root.nodes = {:?}",
            child_namespaces
        );
        assert!(
            child_namespaces.contains(&s!("child-c")),
            "child-c missing; root.nodes = {:?}",
            child_namespaces
        );

        // child-c's own non-shared `local` function survives the
        // promotion pass with shared = false.
        let child_c = topology
            .nodes
            .get("child-c")
            .expect("child-c node present");
        let local = child_c
            .functions
            .get("local")
            .expect("child-c retains its own `local` function after promotion");
        assert!(
            !local.shared,
            "child-c's own `local` function must have shared = false"
        );

        // Roles invariant: every non-`provided` role referenced by
        // `root.functions` must live in `root.roles` after promotion.
        // Before the role-recompute fix, root.roles was computed from
        // root.functions *before* promotion (so it didn't contain
        // shared functions' roles), which would cause `deployer::create`
        // to skip provisioning those roles and AWS to reject the
        // shared Lambdas' creation. This assertion is the regression
        // guard.
        for (fname, f) in &topology.functions {
            if f.runtime.role.kind.to_str() != "provided" {
                assert!(
                    topology.roles.contains_key(&f.runtime.role.name),
                    "role `{}` for function `{}` must be present in root.roles \
                     after promotion; root.roles keys = {:?}",
                    &f.runtime.role.name,
                    fname,
                    topology.roles.keys().collect::<Vec<_>>()
                );
            }
        }
        // Symmetrically, no descendant's `roles` should reference a
        // function it no longer owns. For each descendant, every entry
        // in its `roles` map must correspond to either (a) one of its
        // own functions, or (b) its own state-machine flow's role.
        // Stale entries — most commonly the shared functions' roles
        // left behind when their functions were promoted away — would
        // cause each node deploy to redundantly create-or-update
        // roles already provisioned by root.
        fn assert_roles_match_functions(t: &Topology, path: &str) {
            for (rname, _) in &t.roles {
                let owned_by_function = t
                    .functions
                    .values()
                    .any(|f| &f.runtime.role.name == rname);
                let owned_by_flow = t
                    .flow
                    .as_ref()
                    .map(|fl| &fl.role.name == rname)
                    .unwrap_or(false);
                assert!(
                    owned_by_function || owned_by_flow,
                    "stale role `{}` in {}/roles after promotion (no matching \
                     function or flow); functions = {:?}",
                    rname,
                    path,
                    t.functions.keys().collect::<Vec<_>>()
                );
            }
            for (name, child) in &t.nodes {
                assert_roles_match_functions(child, &format!("{path}/{name}"));
            }
        }
        for (name, child) in &topology.nodes {
            assert_roles_match_functions(child, name);
        }

        // Resolver-cache backwards-compat: a Function serialized to
        // JSON with the `shared` key stripped (legacy entry) must
        // deserialize cleanly with shared = false. Validates the
        // `#[serde(default)]` attribute on Function.shared.
        let foo = topology.functions.get("foo").unwrap();
        let mut value = serde_json::to_value(foo).expect("Function serializes to JSON");
        let removed = value
            .as_object_mut()
            .expect("Function JSON is an object")
            .remove("shared");
        assert!(
            removed.is_some(),
            "freshly serialized Function must include `shared` key"
        );
        let legacy: Function = serde_json::from_value(value)
            .expect("Function JSON missing `shared` must deserialize via #[serde(default)]");
        assert!(
            !legacy.shared,
            "missing `shared` field must default to false on legacy resolver-cache JSON"
        );
    }

    /// `intern_functions` reads the topology spec's `functions:` block.
    /// For inline declarations whose `uri:` starts with `.` (relative),
    /// the produced `Function` must carry `shared = true`. For inline
    /// declarations without a `uri:`, or with one that doesn't start
    /// with `.`, the function is treated as a normal in-tree function
    /// and must carry `shared = false`.
    ///
    /// Exercised end-to-end via the fixture above (each child's
    /// `foo`/`bar` is shared, `child-c/local` is not). This stub
    /// directly inspects the fixture's outputs without a separate
    /// fixture for clarity / fewer dependencies on `TopologySpec`
    /// internals (`TopologySpec` does not implement `Default`, so a
    /// pure unit test would have to manually populate every field).
    #[test]
    fn intern_marks_relative_uri_imports_as_shared() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();
        write_topology_yml(root, "name: parent\nkind: step-function\n");
        write_shared_function(&root.join("shared/x"), "x", "shared_x");
        write_topology_yml(
            &root.join("a"),
            "name: child-a\nkind: step-function\nfunctions:\n  x:\n    uri: ../shared/x\n",
        );

        let topology = Topology::new(root.to_str().unwrap(), true, false);
        let promoted = topology
            .functions
            .get("x")
            .expect("relatively-imported function must end up at root");
        assert!(
            promoted.shared,
            "relative-uri import must have shared = true after promotion"
        );
    }
}
