use super::function::layer;
use super::function::layer::Layer;
use colored::Colorize;
use ptree::builder::TreeBuilder;
use ptree::item::StringItem;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use configurator::Config;
use serde_json::Value;

use super::spec::{TopologySpec, TopologyKind};
use super::{mutation, schedule, event, version, template};
use super::mutation::Mutation;
use super::function::Function;
use super::route::Route;
use super::event::Event;
use super::queue::Queue;
use super::log::LogConfig;
use super::schedule::Schedule;
use super::flow::Flow;
use super::role::Role;
use kit as u;
use kit::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Topology {
    pub namespace: String,
    pub env: String,
    pub fqn: String,
    pub kind: TopologyKind,
    pub nodes: Vec<Topology>,
    pub infra: String,
    pub dir: String,
    pub sandbox: String,
    pub hyphenated_names: bool,
    pub version: String,
    pub events: HashMap<String, Event>,
    pub routes: HashMap<String, Route>,
    pub functions: HashMap<String, Function>,
    pub mutations: HashMap<String, Mutation>,
    pub schedules: HashMap<String, Schedule>,
    pub queues: HashMap<String, Queue>,
    pub logs: LogConfig,
    pub flow: Option<Flow>
}

fn relative_root_path() -> (String, String) {
    let cur = u::pwd();
    let root = u::split_first(&cur, "/services/");
    let next = u::second(&cur, "/services/");
    (root, next)
}

fn legacy_infra_dir(namespace: &str) -> Option<String> {
    u::any_path(vec![
        format!("../../../infrastructure/tc/{}", namespace),
        format!("../../infrastructure/tc/{}", namespace),
        format!("../infrastructure/tc/{}", namespace),
        format!("infrastructure/tc/{}", namespace),
        format!("infra/{}", namespace),
        s!("infra"),
    ])
}

fn as_infra_dir(given_infra_dir: Option<String>, namespace: &str) -> String {
    match given_infra_dir {
        Some(d) => d,
        None => {
            let legacy_dir = legacy_infra_dir(namespace);

            match legacy_dir {
                Some(p) => p,
                None => {
                    let (root, next) = relative_root_path();
                    format!("{root}/infrastructure/tc/{next}")
                }
            }
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

pub fn is_relative_topology_dir(dir: &str) -> bool {
    let topology_file = parent_topology_file(dir);
    match topology_file {
        Some(file) => Path::new(&file).exists(),
        None => false,
    }
}


// functions
fn is_standalone_function_dir(dir: &str) -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    let parent_file = match parent_topology_file(dir) {
        Some(file) => file,
        None => u::empty(),
    };
    u::file_exists(function_file) && !u::file_exists(topology_file) && !u::file_exists(&parent_file)
        || u::file_exists("handler.rb")
        || u::file_exists("handler.py")
        || u::file_exists("main.go")
        || u::file_exists("Cargo.toml")
        || u::file_exists("handler.janet")
        || u::file_exists("handler.clj")
        || u::file_exists("handler.js")
        || u::file_exists("main.janet")
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    u::file_exists(function_file) && u::file_exists(topology_file)
}

fn intern_functions(root_dir: &str, infra_dir: &str, spec: &TopologySpec) -> HashMap<String, Function> {
    let shared = &spec.functions.shared;
    let namespace = &spec.name;

    let mut functions: HashMap<String, Function> = HashMap::new();
    for d in shared {
        let abs_dir = u::absolute_dir(root_dir, &d);
        if u::is_dir(&abs_dir) {
            let function = Function::new(&abs_dir, infra_dir, &namespace, spec.fmt());
            functions.insert(abs_dir, function);
        }
    }
    functions
}

fn function_dirs(dir: &str) -> Vec<String> {
    let known_roots = vec!["resolvers", "functions", "backend"];
    let mut dirs: Vec<String> = u::list_dir(dir);
    for root in known_roots {
        let mut xs = u::list_dir(root);
        dirs.append(&mut xs)
    }
    if path_exists(dir, "function.json") {
        dirs.push(dir.to_string())
    }
    dirs
}

fn ignore_function(dir: &str, root_dir: &str) -> bool {
    let ignore_file = u::path_of(root_dir, ".tcignore");
    if dir.contains(".circleci") || dir.contains(".git") || dir.contains(".vendor") {
        return true
    }
    if u::file_exists(&ignore_file) {
        let globs = u::readlines(&ignore_file);
        for g in globs {
            if u::is_dir(&g) && dir.ends_with(&g) {
                return true
            } else {
                continue
            }
         }
        return false
    } else {
        false
    }
}

fn discover_functions(dir: &str, infra_dir: &str, spec: &TopologySpec) -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    let dirs = function_dirs(dir);
    tracing::debug!("Compiling functions");
    for d in dirs {
        if u::is_dir(&d) && !ignore_function(&d, dir) {
            let function = Function::new(&d, infra_dir, &spec.name, spec.fmt());
            functions.insert(d, function);
        }
    }
    functions
}

fn current_function(dir: &str, infra_dir: &str, spec: &TopologySpec) -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    if u::is_dir(dir) && !dir.starts_with(".") {
        let function = Function::new(dir, infra_dir, &spec.name, spec.fmt());
        functions.insert(dir.to_string(), function);
    }
    functions
}

// nodes

fn should_ignore_node(
    root_dir: &str,
    ignore_nodes: Vec<String>,
    topology_dir: &str
) -> bool {

    let ignore_file = u::path_of(root_dir, ".tcignore");
    if u::file_exists(&ignore_file) {
        let globs = u::readlines(&ignore_file);
        for g in globs {
            let gdir = format!("{}/{}", root_dir, &g);
            if topology_dir.ends_with(&g) || topology_dir.contains(&gdir) {
                return true
            } else {
                continue
            }
        }
        return false
    } else {
        for node in ignore_nodes {
            let abs_path = format!("{root_dir}/{node}");
            if &abs_path == topology_dir {
                return true
            }
            if topology_dir.starts_with(&abs_path) {
                return true
            }
            return false
        }
        return false
    }
}

fn discover_leaf_nodes(root_dir: &str, infra_dir: &str, dir: &str, spec: &TopologySpec) -> Vec<Topology> {
    let ignore_nodes = &spec.nodes.ignore;

    let mut nodes: Vec<Topology> = vec![];
    if is_topology_dir(dir) {
        if !should_ignore_node(root_dir, ignore_nodes.clone(), dir) {
            let f = format!("{}/topology.yml", dir);
            let spec = TopologySpec::new(&f);
            let mut functions = discover_functions(dir, infra_dir, &spec);
            let interned = intern_functions(dir, infra_dir, &spec);
            functions.extend(interned);
            let node = make(root_dir, dir, &spec, functions, vec![]);
            nodes.push(node);
        }
    }
    nodes
}

pub fn discover_nodes(root_dir: &str, infra_dir: &str, spec: &TopologySpec) -> Vec<Topology> {
    let ignore_nodes = &spec.nodes.ignore;
    let dir = u::pwd();
    let mut nodes: Vec<Topology> = vec![];
    for entry in WalkDir::new(dir.clone())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if is_topology_dir(&p) && dir.clone() != p.clone() {
            if !should_ignore_node(root_dir, ignore_nodes.clone(), &p) {
                let f = format!("{}/topology.yml", &p);
                let spec = TopologySpec::new(&f);
                tracing::debug!("Compiling node {}", &spec.name);
                let mut functions = discover_functions(&p, infra_dir, &spec);
                let interned = intern_functions(&p, infra_dir, &spec);
                functions.extend(interned);
                let leaf_nodes = discover_leaf_nodes(root_dir, infra_dir, &p, &spec);
                let node = make(root_dir, &p, &spec, functions, leaf_nodes);
                nodes.push(node);
            }
        }
    }
    nodes
}

// builders
fn make_events(spec: &TopologySpec, fqn: &str, config: &Config) -> HashMap<String, Event> {
    let events = &spec.events;
    let mut h: HashMap<String, Event> = HashMap::new();
    if let Some(evs) = events {
        if let Some(c) = &evs.consumes {
            tracing::debug!("Compiling events");
            for (name, ev) in c.clone().into_iter() {
                let targets = event::make_targets(&name, ev.function, ev.mutation, ev.stepfunction, fqn);
                let ev = Event::new(&name, ev.rule_name, &ev.producer, ev.filter,
                                    ev.pattern, targets, ev.sandboxes, config);
                h.insert(name, ev);
            }
        }
    }
    h
}

fn make_routes(spec: &TopologySpec, config: &Config) -> HashMap<String, Route> {
    let routes = &spec.routes;
    match routes {
        Some(xs) => {
            tracing::debug!("Compiling routes");
            let mut h: HashMap<String, Route> = HashMap::new();
            for (name, rspec) in xs {
                let route = Route::new(rspec, config);
                h.insert(name.to_string(), route);
            }
            h
        },
        None => HashMap::new()
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

fn find_kind(given_kind: &Option<TopologyKind>, flow: &Option<Flow>) -> TopologyKind {
    match given_kind {
        Some(k) => k.clone(),
        None => match flow {
            Some(_) => TopologyKind::StepFunction,
            None => TopologyKind::Function
        }
    }
}

fn make(
    root_dir: &str,
    dir: &str,
    spec: &TopologySpec,
    functions: HashMap<String, Function>,
    nodes: Vec<Topology>,
) -> Topology {

    let config = Config::new(None, "{{env}}");

    let mut functions = functions;
    let namespace = spec.name.to_owned();
    let infra_dir = as_infra_dir(spec.infra.to_owned(), &spec.name);
    let interned = intern_functions(root_dir, &infra_dir, &spec);
    functions.extend(interned);

    let version = version::current_semver(&namespace);
    let fqn = template::topology_fqn(&namespace, spec.hyphenated_names);
    let flow = Flow::new(dir, &infra_dir, &fqn, &spec);

    Topology {
        namespace: namespace.clone(),
        fqn: fqn.clone(),
        env: template::profile(),
        kind: find_kind(&spec.kind, &flow),
        version: version,
        infra: infra_dir.to_owned(),
        sandbox: template::sandbox(),
        dir: s!(dir),
        hyphenated_names: spec.hyphenated_names.to_owned(),
        nodes: nodes,
        functions: functions,
        events: make_events(&spec, &fqn, &config),
        schedules: schedule::make_all(&infra_dir),
        routes: make_routes(&spec, &config),
        queues: make_queues(&spec, &config),
        mutations: make_mutations(&spec, &config),
        logs: LogConfig::new(),
        flow: flow
    }
}

fn make_relative(dir: &str) -> Topology {
    let f = match parent_topology_file(dir) {
        Some(file) => file,
        None => format!("../topology.yml"),
    };

    let spec = TopologySpec::new(&f);
    let namespace = &spec.name;
    let infra_dir = as_infra_dir(spec.infra.to_owned(), &spec.name);
    let function = Function::new(dir, &infra_dir, namespace, &spec.fmt());
    let functions = Function::to_map(function);
    let nodes = vec![];
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
        version: version::current_semver(&namespace),
        sandbox: template::sandbox(),
        infra: u::empty(),
        dir: s!(dir),
        hyphenated_names: false,
        events: HashMap::new(),
        routes: HashMap::new(),
        flow: None,
        functions: functions,
        nodes: vec![],
        mutations: HashMap::new(),
        queues: HashMap::new(),
        logs: LogConfig::new(),
        schedules: HashMap::new(),
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
            make(dir, dir, &spec, functions, vec![])

        } else if is_topology_dir(dir) {

            let f = format!("{}/topology.yml", dir);
            let spec = TopologySpec::new(&f);
            let infra_dir = as_infra_dir(spec.infra.to_owned(), &spec.name);
            let nodes;
            if recursive {
                nodes = discover_nodes(dir, &infra_dir, &spec);
            } else {
                nodes = vec![];
            }
            if skip_functions {
                let functions = HashMap::new();
                make(dir, dir, &spec, functions, nodes)
            } else {
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
        for node in &self.nodes {
            fns.extend(node.clone().functions);
        }
        fns.clone()
    }

    pub fn build_tree(&self) -> StringItem {
        let mut t = TreeBuilder::new(s!(self.namespace.blue()));

        for (_, f) in &self.functions {
            t.begin_child(s!(f.name.green()));
            t.add_empty_child(f.runtime.lang.to_str());
            t.add_empty_child(f.runtime.role.path.to_string());
            t.end_child();
        }

        for node in &self.nodes {
            t.begin_child(s!(&node.namespace.green()));
            for (_, f) in &node.functions {
                t.begin_child(s!(&f.fqn));
                t.add_empty_child(f.runtime.lang.to_str());
                t.add_empty_child(f.runtime.role.path.to_string());
                t.end_child();
            }
            t.end_child();
        }

        t.build()
    }

    pub fn layers(&self) -> Vec<Layer> {
        let fns = self.functions();
        layer::find(fns)
    }

    pub fn to_str(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn roles(&self) -> Vec<Role> {
        let mut xs: Vec<Role> = vec![];
        for (_, f) in &self.functions {
            if &f.runtime.role.path != "provided" {
                xs.push(f.runtime.role.clone())
            }
        }
        xs
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
