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

            let function = Function::new(&abs_dir, infra_dir, &namespace, spec.fmt());
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

/// Compose a function-only topology when `tc` is invoked from inside a
/// function-level subdirectory of a topology (e.g. a directory laid
/// out as `<repo>/topologies/<parent>/<function>/`).
///
/// The parent `topology.yml` is read **only** for the things that scope
/// this lambda to its parent: the namespace prefix used in the
/// function's fqn (so the lambda is named e.g.
/// `<parent>_<function>_<sandbox>` exactly as it would be from a
/// topology-level deploy), the infra-dir convention, and the spec
/// format (hyphenated vs. underscored fqn). Crucially, the returned
/// `Topology`:
///
/// - has `flow: None` — running `tc` from a sub-function dir is a
///   per-function deploy and must **not** create or update the parent
///   state machine,
/// - carries only the function in this directory in `functions` —
///   inline functions declared on the parent are deliberately **not**
///   intern'd here, since the user's intent is to deploy this one
///   lambda,
/// - has empty `events`, `routes`, `mutations`, `queues`, `channels`,
///   `pages`, `pools`, `nodes` — the parent's wider topology shape is
///   irrelevant to a single-lambda deploy.
///
/// The namespace and resource tags on the resulting topology still come
/// from the parent so that, when the lambda is later seen by a
/// topology-level deploy, its `namespace`/`version` tags match what a
/// topology-level deploy would have written. This preserves continuity
/// with topology-level deploys; the only thing function-only deploys
/// drop is the parent's state-machine update and the parent's wider
/// entity graph.
///
/// History: the previous implementation forwarded the parent's spec to
/// `make()`, which intern'd the parent's `functions:` block via
/// `intern_functions` and synthesized a full state-machine `Flow` from
/// the parent. Because `deployer::create` always creates the state
/// machine when `flow` is `Some(_)` and writes the topology's tags onto
/// it, a function-level deploy ended up rewriting the parent SM and
/// tagging it with the parent's version. Subsequent topology-level
/// deploys at that same version then hit `from == to` in the resolver
/// and silently filtered every declared function out of the deploy
/// set — the state machines came up with no lambdas behind them. The
/// resolver-side half of the fix is in `lib/resolver/src/function.rs`
/// (`deployment_lookup_target`); this is the composer-side half that
/// prevents the corruption from being introduced in the first place.
fn make_relative(dir: &str) -> Topology {
    let parent_yml = match parent_topology_file(dir) {
        Some(file) => file,
        None => format!("../topology.yml"),
    };

    let parent_spec = TopologySpec::new(&parent_yml);
    let parent_namespace = parent_spec.name.clone();
    let infra_dir = as_infra_dir(parent_spec.infra.to_owned(), dir);

    // Compose the function from its own dir, prefixed with the parent's
    // namespace so its AWS lambda name (e.g. `<parent>_<function>_<sandbox>`)
    // matches what a topology-level deploy would produce.
    let function = Function::new(dir, &infra_dir, &parent_namespace, &parent_spec.fmt());
    let functions = Function::to_map(function.clone());

    Topology {
        namespace: parent_namespace.clone(),
        env: template::profile(),
        // The topology fqn here is the *function's* fqn — so the
        // resolver's deployment-version lookup
        // (`find_modified` -> `snapshotter::find_version`) consults the
        // lambda's own resource tags, not the parent state machine's.
        fqn: function.fqn.clone(),
        kind: TopologyKind::Function,
        concurrent: false,
        version: u::current_semver(&parent_namespace),
        sandbox: template::sandbox(),
        infra: u::gdir(&infra_dir),
        dir: dir.to_string(),
        hyphenated_names: parent_spec.hyphenated_names,
        // No parent state machine, no parent inline functions, no
        // parent events/routes/etc. The topology-shaped fields below
        // are deliberately empty — a per-function deploy must not
        // touch any of them.
        nodes: HashMap::new(),
        events: HashMap::new(),
        routes: HashMap::new(),
        mutations: HashMap::new(),
        schedules: HashMap::new(),
        queues: HashMap::new(),
        channels: HashMap::new(),
        pools: HashMap::new(),
        pages: HashMap::new(),
        flow: None,
        roles: make_roles(&functions, &0, &0, &0, &None),
        base_roles: make_base_roles(),
        functions,
        tags: tag::make(&parent_namespace, &infra_dir),
        tests: HashMap::new(),
        config: Config::new(),
        transducer: None,
        sequences: HashMap::new(),
    }
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

            let nodes;
            if recursive {
                tracing::debug!("Recursive {}", dir);
                nodes = make_nodes(dir, &spec);
            } else {
                nodes = HashMap::new();
            }
            if skip_functions {
                tracing::debug!("Skipping functions {}", dir);
                let functions = HashMap::new();
                make(dir, dir, &spec, functions, nodes)
            } else {
                tracing::debug!("Discovering functions {}", dir);
                let functions = discover_functions(dir, &infra_dir, &spec);
                make(dir, dir, &spec, functions, nodes)
            }
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
mod make_relative_tests {
    //! Regression tests for the function-dir deploy bug: running
    //! `tc create` / `tc update` from inside a function-level
    //! subdirectory of a topology must produce a function-only
    //! `Topology`, not a synthesized full topology that would re-deploy
    //! the parent state machine and the parent's inline functions.
    //!
    //! Each test sets up a tempdir laid out like
    //!
    //!   <tmp>/topologies/<parent>/topology.yml
    //!   <tmp>/topologies/<parent>/<func>/handler.py
    //!
    //! and calls the public `Topology::new(child_dir, true, false)`
    //! entry point so we exercise the same routing
    //! (`is_relative_topology_dir` -> `make_relative`) that production
    //! goes through.
    //!
    //! We do not need a git repo for these tests: `Topology::new` does
    //! not require one, and `current_semver` falls back to `"0.0.1"`
    //! when no matching tag is reachable from HEAD.

    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Generic placeholder names used across these tests. They are
    /// intentionally abstract so the tests document the contract
    /// without being coupled to any one repo's topology layout.
    const PARENT: &str = "parent-topology";
    const CHILD_FN: &str = "child-function";
    const INLINE_FN_A: &str = "inline-fn-a";
    const INLINE_FN_B: &str = "inline-fn-b";

    fn write(root: &Path, rel: &str, contents: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, contents).unwrap();
    }

    /// Build a tmpdir layout where the parent topology declares both
    /// inline `functions:` AND a state-machine `flow`, with a child
    /// function dir under it. This is the shape that exercises the
    /// historic bug: any code path that re-pulls the parent's spec
    /// (`intern_functions`, `Flow::new`) would observe non-empty
    /// values and write them into the function-only topology.
    ///
    /// Returns `(tempdir, child_dir_path)`.
    fn parent_with_inline_fns_and_flow_plus_child(
        parent_name: &str,
        child_name: &str,
    ) -> (TempDir, String) {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();

        let parent_rel = format!("topologies/{}", parent_name);
        let child_rel = format!("{}/{}", parent_rel, child_name);

        let parent_yml = format!(
            r#"name: {parent_name}
kind: state-machine
mode: Standard
version: "0.0.1"

functions:
  {inline_a}:
    uri: ./{inline_a}
  {inline_b}:
    uri: ./{inline_b}

flow:
  Comment: parent state machine
  StartAt: do-thing
  States:
    do-thing:
      Type: Pass
      End: true
"#,
            parent_name = parent_name,
            inline_a = INLINE_FN_A,
            inline_b = INLINE_FN_B,
        );
        write(root, &format!("{}/topology.yml", parent_rel), &parent_yml);

        // The two inline-function dirs referenced by the parent. We
        // create them as plausible function dirs so that, if the bug
        // were re-introduced and the composer tried to intern them, it
        // could find files there.
        write(
            root,
            &format!("{}/{}/handler.py", parent_rel, INLINE_FN_A),
            "",
        );
        write(
            root,
            &format!("{}/{}/handler.py", parent_rel, INLINE_FN_B),
            "",
        );

        // The child function dir we'll run `tc` from.
        write(root, &format!("{}/handler.py", child_rel), "");

        let child_dir = root.join(&child_rel).to_str().unwrap().to_string();
        (tmp, child_dir)
    }

    /// THE primary regression test for the function-dir deploy bug.
    /// Running tc against a function-level subdirectory must not
    /// synthesize a state-machine `flow`. The deployer creates a state
    /// machine when `flow` is `Some(_)`, and that side effect was what
    /// rewrote the parent SM's version tag during a function-only
    /// deploy.
    #[test]
    fn make_relative_does_not_pull_in_parent_flow() {
        let (_tmp, child_dir) = parent_with_inline_fns_and_flow_plus_child(PARENT, CHILD_FN);

        let topology = Topology::new(&child_dir, true, false);

        assert!(
            topology.flow.is_none(),
            "make_relative must NOT synthesize the parent's state-machine flow. \
             A function-level deploy that produces flow=Some triggers \
             deployer::state::create on the parent state machine and rewrites \
             its version tag — the corruption that produces the silent-no-op \
             pattern documented in lib/resolver/src/function.rs."
        );
    }

    /// Companion to the flow test: the parent's inline `functions:`
    /// block must not be intern'd into the function-only topology. If
    /// it were, deployer::function::create would deploy unrelated
    /// lambdas that the user did not ask for.
    #[test]
    fn make_relative_does_not_pull_in_parent_inline_functions() {
        let (_tmp, child_dir) = parent_with_inline_fns_and_flow_plus_child(PARENT, CHILD_FN);

        let topology = Topology::new(&child_dir, true, false);

        // The only function in the topology should be the one in the
        // dir we ran from.
        assert_eq!(
            topology.functions.len(),
            1,
            "make_relative must produce a topology containing exactly the \
             function in the dir we ran from, not the parent's inline \
             functions. Got: {:?}",
            topology.functions.keys().collect::<Vec<_>>()
        );
        assert!(
            topology.functions.contains_key(CHILD_FN),
            "expected only `{}`; got: {:?}",
            CHILD_FN,
            topology.functions.keys().collect::<Vec<_>>()
        );
        assert!(
            !topology.functions.contains_key(INLINE_FN_A),
            "parent's inline `{}` must not be pulled in",
            INLINE_FN_A
        );
        assert!(
            !topology.functions.contains_key(INLINE_FN_B),
            "parent's inline `{}` must not be pulled in",
            INLINE_FN_B
        );
    }

    /// `topology.kind` for a function-only deploy must be `Function`,
    /// not the parent's `StepFunction`. Among other things this drives
    /// `find_modified` to look up the lambda's resource tags via the
    /// Lambda API rather than the State Machines API.
    #[test]
    fn make_relative_kind_is_function_not_step_function() {
        let (_tmp, child_dir) = parent_with_inline_fns_and_flow_plus_child(PARENT, CHILD_FN);

        let topology = Topology::new(&child_dir, true, false);

        assert!(
            matches!(topology.kind, TopologyKind::Function),
            "make_relative must mark the topology as Function-kind so the \
             resolver's deployment-version lookup hits the Lambda API, not \
             Step Functions. Got kind={:?}",
            topology.kind
        );
    }

    /// `topology.fqn` must be the *function's* fqn (e.g.
    /// `<parent>_<function>_{{sandbox}}`), not the parent's
    /// (`<parent>_{{sandbox}}`). Together with the resolver-side fix
    /// in `find_modified`, this is what makes the deployment lookup
    /// consult the right AWS resource (the lambda) when running from
    /// a function dir.
    #[test]
    fn make_relative_fqn_is_function_fqn_not_parent_fqn() {
        let (_tmp, child_dir) = parent_with_inline_fns_and_flow_plus_child(PARENT, CHILD_FN);

        let topology = Topology::new(&child_dir, true, false);

        let expected = format!("{}_{}_{{{{sandbox}}}}", PARENT, CHILD_FN);
        assert_eq!(
            topology.fqn, expected,
            "topology.fqn must point at the function we're deploying, not at \
             the parent state machine. The previous implementation set this \
             to the parent's fqn (e.g. '{}_{{{{sandbox}}}}') which made the \
             resolver look up the parent SM's deployed-version tag instead \
             of the lambda's.",
            PARENT
        );
    }

    /// A function-only deploy has no nested topologies. This locks in
    /// that `Topology::new(child_dir, recursive=true, ...)` does NOT
    /// walk the parent's directory tree synthesizing nested nodes.
    #[test]
    fn make_relative_has_no_nested_nodes() {
        let (_tmp, child_dir) = parent_with_inline_fns_and_flow_plus_child(PARENT, CHILD_FN);

        let topology = Topology::new(&child_dir, true, false);

        assert!(
            topology.nodes.is_empty(),
            "make_relative must produce zero nodes even when called with \
             recursive=true; got {:?}",
            topology.nodes.keys().collect::<Vec<_>>()
        );
    }

    /// A function-only deploy must carry empty `events`, `routes`,
    /// `mutations`, `queues`, `channels`, `pages`, `pools`. If any of
    /// these were non-empty, `deployer::create` would touch the
    /// corresponding AWS resources for the parent topology — exactly
    /// the topology-level side effect that running tc from a function
    /// dir must not have.
    #[test]
    fn make_relative_drops_all_topology_level_entities() {
        let (_tmp, child_dir) = parent_with_inline_fns_and_flow_plus_child(PARENT, CHILD_FN);

        let topology = Topology::new(&child_dir, true, false);

        assert!(topology.events.is_empty(), "events must be empty");
        assert!(topology.routes.is_empty(), "routes must be empty");
        assert!(topology.mutations.is_empty(), "mutations must be empty");
        assert!(topology.queues.is_empty(), "queues must be empty");
        assert!(topology.channels.is_empty(), "channels must be empty");
        assert!(topology.pages.is_empty(), "pages must be empty");
        assert!(topology.pools.is_empty(), "pools must be empty");
    }

    /// The function's dir is preserved as `topology.dir`. This is what
    /// `try_update`'s `current_function(pwd)` lookup matches against
    /// — get this wrong and `tc update` from a function dir silently
    /// degrades to topology-level update.
    #[test]
    fn make_relative_topology_dir_is_function_dir() {
        let (_tmp, child_dir) = parent_with_inline_fns_and_flow_plus_child(PARENT, CHILD_FN);

        let topology = Topology::new(&child_dir, true, false);

        assert_eq!(
            topology.dir, child_dir,
            "topology.dir must equal the function dir we ran from so that \
             try_update's current_function(pwd) match works correctly"
        );
    }
}
