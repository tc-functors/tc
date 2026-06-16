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
    index,
    sequence,
    tag,
    version
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
    /// Full set of this topology's own functions, populated by the
    /// composer and **never replaced by the resolver**. The resolver
    /// shrinks `functions` to the modified subset (for code uploads),
    /// but role/policy reconciliation needs to operate over every
    /// function so that adding/changing a per-function role JSON
    /// re-attaches the new role to the lambda even when its code
    /// hasn't changed.
    #[serde(default)]
    pub all_functions: HashMap<String, Function>,
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
    let idx = index::get();
    if idx.covers(Path::new(dir)) {
        return idx.is_topology_dir(dir);
    }
    let topology_file = format!("{}/topology.yml", dir);
    Path::new(&topology_file).exists()
}

fn parent_topology_file(dir: &str) -> Option<String> {
    u::find_parent_file(dir, "topology.yml")
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

fn is_shared(uri: Option<String>, given_shared: Option<bool>) -> bool {
    if let Some(is) = given_shared {
        is
    } else {
        match uri {
            Some(p) => p.starts_with("."),
            None => false,
        }
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
        if is_shared(f.uri.clone(), f.shared) {
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
    let idx = index::get();
    if idx.covers(Path::new(dir)) {
        return idx.is_inferred_dir(dir);
    }
    u::path_exists(dir, "handler.rb")
        || u::path_exists(dir, "handler.py")
        || u::path_exists(dir, "main.go")
        || u::path_exists(dir, "Cargo.toml")
        || u::path_exists(dir, "handler.janet")
        || u::path_exists(dir, "handler.clj")
        || u::path_exists(dir, "handler.js")
        || u::path_exists(dir, "main.janet")
}

fn function_dirs(dir: &str, function_dirs: Option<Vec<String>>) -> Vec<String> {
    let mut known_roots = vec![s!("resolvers"), s!("functions"), s!("transformers")];
    if let Some(dirs) = function_dirs {
        known_roots.extend(dirs)
    }

    let mut xs: Vec<String> = vec![];
    let idx = index::get();
    let covered = idx.covers(Path::new(dir));
    let dirs = if covered {
        idx.list_subdirs(dir)
    } else {
        u::list_dirs(dir)
    };
    for root in known_roots {
        let mut xm = if covered {
            idx.list_subdirs(&root)
        } else {
            u::list_dirs(&root)
        };
        xs.append(&mut xm)
    }
    for d in dirs {
        let has_marker = if covered {
            idx.path_exists(&d, "function.yml")
                || idx.path_exists(&d, "function.json")
                || idx.is_inferred_dir(&d)
        } else {
            path_exists(&d, "function.yml")
                || path_exists(&d, "function.json")
                || is_inferred_dir(&d)
        };
        if has_marker {
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
    function_dirs(dir, spec.function_dirs.clone())
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
    let dirs: Vec<String> = function_dirs(dir, spec.function_dirs.clone())
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
    // `root_dir` arrives in whatever form the caller had on hand
    // (typically `kit::pwd()`, which on macOS keeps `/tmp/...` rather
    // than canonicalising to `/private/tmp/...`). `topology_dir` is
    // produced by `nested_topology_dirs` via the `composer::index`,
    // which keys on canonical paths. Without normalising both sides,
    // the prefix-equality / `contains` / `starts_with` checks below
    // silently miss any pwd whose path crosses a symlink — `tc`'s
    // `nodes.ignore` and `.tcignore` rules would then fail to match.
    // Falls back to the input strings if canonicalisation fails (e.g.
    // path doesn't exist on disk).
    let canonicalize = |s: &str| -> String {
        Path::new(s)
            .canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| s.to_string())
    };
    let root_dir_owned = canonicalize(root_dir);
    let topology_dir_owned = canonicalize(topology_dir);
    let root_dir = root_dir_owned.as_str();
    let topology_dir = topology_dir_owned.as_str();

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

/// Find every nested topology dir under `root_dir`. Uses the
/// process-wide [`index`] when `root_dir` is covered by it (the common
/// case during `tc compose` / `tc diff`); falls back to a fresh
/// `WalkDir` for callers that target a dir outside the indexed pwd.
fn nested_topology_dirs(root_dir: &str) -> Vec<String> {
    let idx = index::get();
    if idx.covers(Path::new(root_dir)) {
        let canonical_root = match Path::new(root_dir).canonicalize() {
            Ok(p) => p,
            Err(_) => return vec![],
        };
        let mut out: Vec<String> = idx
            .descendants_of(&canonical_root)
            .filter(|(p, info)| *p != canonical_root.as_path() && info.has("topology.yml"))
            .filter_map(|(p, _)| p.to_str().map(|s| s.to_string()))
            .collect();
        out.sort();
        return out;
    }
    WalkDir::new(root_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_string_lossy().to_string())
        .filter(|p| is_topology_dir(p) && root_dir != p)
        .collect()
}

fn make_nodes(root_dir: &str, spec: &TopologySpec) -> HashMap<String, Topology> {
    let ignore_nodes = &spec.nodes.ignore;
    let candidates: Vec<String> = nested_topology_dirs(root_dir)
        .into_iter()
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

    let version = version::current_semver(&namespace);
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
        all_functions: functions.clone(),
        functions: functions,
        schedules: schedule::make_all(&namespace, &infra_dir),
        queues: queues,
        mutations: mutations,
        channels: channels,
        pools: make_pools(&spec, &config),
        tags: tag::make(&spec.name, &infra_dir),
        pages: page::make_all(dir, &spec, &infra_dir, &config),
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
        all_functions: functions.clone(),
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

/// Recursively drains `shared` functions from `node` and descendants into
/// `target`, deduped by key (first wins). Returns the number of duplicates
/// encountered (already present in `target`).
fn drain_shared_into(target: &mut HashMap<String, Function>, node: &mut Topology) -> usize {
    let mut duplicates = 0usize;
    for child in node.nodes.values_mut() {
        duplicates += drain_shared_into(target, child);
    }
    let drained = std::mem::take(&mut node.functions);
    let mut owned: HashMap<String, Function> = HashMap::with_capacity(drained.len());
    for (name, f) in drained {
        if f.shared {
            match target.entry(name.clone()) {
                std::collections::hash_map::Entry::Occupied(existing) => {
                    duplicates += 1;
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
    duplicates
}

/// Recompute `roles` at every level from current `(functions, flow)`.
/// Necessary because `make()` computed roles before promotion relocated
/// shared functions, leaving root missing their roles and descendants
/// holding stale entries.
fn recompute_roles_recursive(t: &mut Topology) {
    for child in t.nodes.values_mut() {
        recompute_roles_recursive(child);
    }
    t.roles = make_roles(&t.functions, &0, &0, &0, &t.flow);
}

/// Drain all `shared` functions from descendants into `root.functions`
/// (first-wins on key collision) and recompute roles at every level.
fn promote_shared_to_root(root: &mut Topology) {
    let mut promoted: HashMap<String, Function> = HashMap::new();
    let mut duplicates = 0usize;
    for child in root.nodes.values_mut() {
        duplicates += drain_shared_into(&mut promoted, child);
    }
    if promoted.is_empty() {
        return;
    }
    let promoted_count = promoted.len();
    for (name, f) in promoted {
        root.functions.entry(name).or_insert(f);
    }
    recompute_roles_recursive(root);
    tracing::info!(
        "Promoted {} shared function(s) to root (eliminated {} duplicate(s))",
        promoted_count,
        duplicates,
    );
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
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    #[test]
    fn should_ignore_node_matches_when_root_reached_via_symlink() {
        let outer = TempDir::new().unwrap();
        let real = outer.path().join("real");
        fs::create_dir_all(real.join("ignore_me")).unwrap();
        fs::create_dir_all(real.join("keep_me")).unwrap();
        let canonical_real = real.canonicalize().unwrap();

        let alias = outer.path().join("link");
        symlink(&real, &alias).unwrap();

        let alias_root = alias.to_str().unwrap();
        let canonical_target = canonical_real.join("ignore_me");
        let canonical_target_str = canonical_target.to_str().unwrap();
        let canonical_keep = canonical_real.join("keep_me");
        let canonical_keep_str = canonical_keep.to_str().unwrap();

        let ignore = Some(vec!["ignore_me".to_string()]);
        assert!(
            should_ignore_node(alias_root, ignore.clone(), canonical_target_str),
            "ignore rule should fire even though root_dir uses an aliased path"
        );
        assert!(
            !should_ignore_node(alias_root, ignore, canonical_keep_str),
            "non-matching dir should not be ignored"
        );
    }

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
        fs::write(dir.join("handler.py"), "").unwrap();
    }

    fn write_topology_yml(dir: &std::path::Path, contents: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("topology.yml"), contents).unwrap();
    }

    #[test]
    fn compose_recursive_dedups_shared_functions_to_root() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();

        write_topology_yml(
            root,
            "name: shared-dedup-parent\nkind: step-function\n",
        );

        write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");
        write_shared_function(&root.join("shared/bar"), "bar", "shared_bar");

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
        write_shared_function(&root.join("c/local"), "local", "child_c_local");

        let topology = Topology::new(root.to_str().unwrap(), true, false);

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
    }

    #[test]
    fn shared_field_defaults_to_false_on_legacy_json() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();
        write_topology_yml(root, "name: serde-test\nkind: step-function\n");
        write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");
        write_topology_yml(
            &root.join("a"),
            "name: child-a\nkind: step-function\nfunctions:\n  foo:\n    uri: ../shared/foo\n",
        );
        let topology = Topology::new(root.to_str().unwrap(), true, false);

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

    #[test]
    fn root_declaring_shared_function_keeps_it_in_place() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();
        write_shared_function(&root.join("shared/x"), "x", "shared_x");
        write_topology_yml(
            root,
            "name: root-shared\nkind: step-function\nfunctions:\n  x:\n    uri: ./shared/x\n",
        );

        let topology = Topology::new(root.to_str().unwrap(), true, false);
        assert!(
            topology.functions.contains_key("x"),
            "root's own shared function must remain in root.functions"
        );
        assert!(
            topology.functions.get("x").unwrap().shared,
            "root's own shared function retains shared = true"
        );
    }

    #[test]
    fn same_source_different_keys_both_promoted() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();
        write_topology_yml(root, "name: parent\nkind: step-function\n");
        write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");
        write_topology_yml(
            &root.join("a"),
            "name: child-a\nkind: step-function\nfunctions:\n  foo:\n    uri: ../shared/foo\n",
        );
        write_topology_yml(
            &root.join("b"),
            "name: child-b\nkind: step-function\nfunctions:\n  my_foo:\n    uri: ../shared/foo\n",
        );

        let topology = Topology::new(root.to_str().unwrap(), true, false);

        assert!(
            topology.functions.contains_key("foo"),
            "key `foo` must be promoted"
        );
        assert!(
            topology.functions.contains_key("my_foo"),
            "key `my_foo` must be promoted (same source, different local name)"
        );
        assert_eq!(
            topology.functions.get("foo").unwrap().dir,
            topology.functions.get("my_foo").unwrap().dir,
            "both keys reference the same source dir"
        );
    }

    #[test]
    fn deep_nesting_shared_functions_promoted() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();
        write_topology_yml(root, "name: parent\nkind: step-function\n");
        write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");

        // Two levels: root -> mid -> leaf, leaf imports shared function
        write_topology_yml(
            &root.join("mid"),
            "name: mid\nkind: step-function\n",
        );
        write_topology_yml(
            &root.join("mid/leaf"),
            "name: leaf\nkind: step-function\nfunctions:\n  foo:\n    uri: ../../shared/foo\n",
        );

        let topology = Topology::new(root.to_str().unwrap(), true, false);

        assert!(
            topology.functions.contains_key("foo"),
            "shared function from deep nesting must be promoted to root"
        );

        fn has_shared_function(t: &Topology, key: &str) -> bool {
            if t.functions.contains_key(key) {
                return true;
            }
            t.nodes.values().any(|child| has_shared_function(child, key))
        }
        for child in topology.nodes.values() {
            assert!(
                !has_shared_function(child, "foo"),
                "no descendant should retain the shared function"
            );
        }
    }

    #[test]
    fn non_recursive_mode_does_not_promote() {
        let outer = TempDir::new().unwrap();
        let root = outer.path();
        write_shared_function(&root.join("shared/x"), "x", "shared_x");
        write_topology_yml(
            root,
            "name: non-recursive\nkind: step-function\nfunctions:\n  x:\n    uri: ./shared/x\n",
        );

        let topology = Topology::new(root.to_str().unwrap(), false, false);

        assert!(
            topology.functions.contains_key("x"),
            "function present in non-recursive mode"
        );
        assert!(
            topology.functions.get("x").unwrap().shared,
            "shared flag is set even in non-recursive mode (marking is unconditional)"
        );
        assert!(
            topology.nodes.is_empty(),
            "non-recursive mode has no child nodes"
        );
    }
}
