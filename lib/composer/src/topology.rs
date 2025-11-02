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
pub use sequence::Connector;
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
    tag,
    sequence,
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
    pub tests: HashMap<String, TestSpec>,
    pub transducer: Option<Transducer>,
    pub sequence: Vec<Connector>
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

fn discover_functions(
    dir: &str,
    infra_dir: &str,
    spec: &TopologySpec,
) -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    let dirs = function_dirs(dir);

    for d in dirs {
        tracing::debug!("function {}", d);
        if u::is_dir(&d) && !ignore_function(&d, dir) {
            let function = Function::new(&d, infra_dir, &spec.name, spec.fmt());
            functions.insert(function.name.clone(), function);
        }
    }
    functions
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
    let mut nodes: HashMap<String, Topology> = HashMap::new();
    for entry in WalkDir::new(root_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if is_topology_dir(&p) && root_dir != p.clone() {
            if !should_ignore_node(root_dir, ignore_nodes.clone(), &p) {
                let f = format!("{}/topology.yml", &p);
                let spec = TopologySpec::new(&f);
                tracing::debug!("node {}", &spec.name);
                let infra_dir = as_infra_dir(spec.infra.to_owned(), &p);
                let mut functions = discover_functions(&p, &infra_dir, &spec);
                let interned = intern_functions(&p, &infra_dir, &spec);
                functions.extend(interned);
                let leaf_nodes = discover_leaf_nodes(&spec.name, root_dir, &p, &spec);
                let node = make(root_dir, &p, &spec, functions, leaf_nodes);
                nodes.insert(spec.name.to_string(), node);
            }
        }
    }
    nodes
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
) -> HashMap<String, Route> {
    let routes = &spec.routes;
    match routes {
        Some(xs) => {
            let mut h: HashMap<String, Route> = HashMap::new();
            for (name, rspec) in xs {
                tracing::debug!("route {}", &name);
                let skip = rspec.doc_only;
                let route = Route::new(fqn, &name, spec, rspec, fns, events, queues, skip);
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
    mutations: &usize,
    routes: &usize,
    events: &usize,
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

    let mut entities: Vec<Entity> = vec![];

    if *mutations > 0 {
        entities.push(Entity::Mutation);
    }

    if *routes > 0 {
        entities.push(Entity::Route);
    }

    if *events > 0 {
        entities.push(Entity::Event);
    }

    if let Some(_f) = states {
        entities.push(Entity::State);
    }

    for b in entities {
        let r = match std::env::var("TC_LEGACY_ROLES") {
            Ok(_) => Role::provided_by_entity(b),
            Err(_) => Role::default(b),
        };
        h.insert(r.name.clone(), r);
    }
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
    let routes = make_routes(&spec, &fqn, &functions, &events, &queues);
    let channels = make_channels(&spec, &config);

    let maybe_transducer = Transducer::new(&namespace, &functions, &events, &mutations, &channels);

    Topology {
        namespace: namespace.clone(),
        fqn: fqn.clone(),
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
        sequence: sequence::make_all(&spec.sequence)
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
        sequence: vec![],
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
