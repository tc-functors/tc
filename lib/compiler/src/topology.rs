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
use super::{mutation, schedule, event, version, template, tag, channel};
use super::mutation::Mutation;
use super::function::Function;
use super::route::Route;
use super::event::Event;
use super::queue::Queue;
use super::log::LogConfig;
use super::schedule::Schedule;
use super::channel::Channel;
use super::flow::Flow;
use super::role::Role;
use super::graph;
use super::graph::Graph;
use kit as u;
use kit::*;

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
    pub tags: HashMap<String, String>,
    pub logs: LogConfig,
    pub flow: Option<Flow>
}

fn relative_root_path(dir: &str) -> (String, String) {
    let root = u::split_first(&dir, "/services/");
    let next = u::second(&dir, "/services/");
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
        spec.nodes.root.is_some()
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
    let function_file = "function.json";
    let topology_file = "topology.yml";
    let parent_file = match parent_topology_file(dir) {
        Some(file) => file,
        None => u::empty(),
    };
    if is_root_topology(&parent_file) {
        return true
    } else {
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
    let known_roots = vec!["resolvers", "functions", "backend"];
    let mut xs: Vec<String> = vec![];
    let dirs = u::list_dirs(dir);
    for root in known_roots {
        let mut xm = u::list_dirs(root);
        xs.append(&mut xm)
    }
    for d in dirs {
        if path_exists(&d, "function.json") || is_inferred_dir(&d) {
            xs.push(d.to_string())
        }
    }
    xs
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
        for node in ignore_nodes.unwrap() {
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

fn discover_leaf_nodes(root_dir: &str, dir: &str, s: &TopologySpec) -> HashMap<String, Topology> {
    let ignore_nodes = &s.nodes.ignore;

    let mut nodes: HashMap<String, Topology> = HashMap::new();
    if is_topology_dir(dir) {
        if !should_ignore_node(root_dir, ignore_nodes.clone(), dir) {
            let f = format!("{}/topology.yml", dir);
            let spec = TopologySpec::new(&f);
            let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
            let mut functions = discover_functions(dir, &infra_dir, &spec);
            let interned = intern_functions(dir, &infra_dir, &spec);
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
                let leaf_nodes = discover_leaf_nodes(root_dir, &p, &spec);
                let node = make(root_dir, &p, &spec, functions, leaf_nodes);
                nodes.insert(spec.name.to_string(), node);
            }
        }
    }
    nodes
}

fn make_events(namespace: &str, spec: &TopologySpec, fqn: &str, config: &Config) -> HashMap<String, Event> {
    let events = &spec.events;
    let mut h: HashMap<String, Event> = HashMap::new();
    if let Some(evs) = events {
        let skip = evs.doc_only;
        if let Some(c) = &evs.consumes {
            for (name, ev) in c.clone().into_iter() {
                tracing::debug!("event {}", &name);
                let targets = event::make_targets(namespace, &name, &ev, fqn);
                let ev = Event::new(&name, &ev, targets, config, skip);
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
            let mut h: HashMap<String, Route> = HashMap::new();
            for (name, rspec) in xs {
                tracing::debug!("route {}", name);
                let route = Route::new(spec, rspec, config);
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

fn make_channels(spec: &TopologySpec, _config: &Config) -> HashMap<String, Channel> {
    match &spec.channels {
        Some(c) => channel::make(&spec.name, c.clone()),
        None => HashMap::new()
    }

}


fn find_kind(
    given_kind: &Option<TopologyKind>,
    flow: &Option<Flow>,
    functions: &HashMap<String, Function>,
    mutations: &HashMap<String, Mutation>
) -> TopologyKind {
    match given_kind {
        Some(k) => k.clone(),
        None => match flow {
            Some(_) => TopologyKind::StepFunction,
            None => {
                if !mutations.is_empty() {
                    return TopologyKind::Graphql
                } else if !functions.is_empty() {
                    return TopologyKind::Function
                } else {
                    return TopologyKind::Evented
                }
            }
        }
    }
}

fn make(
    root_dir: &str,
    dir: &str,
    spec: &TopologySpec,
    functions: HashMap<String, Function>,
    nodes: HashMap<String, Topology>,
) -> Topology {

    let config = Config::new(None, "{{env}}");

    let mut functions = functions;
    let namespace = spec.name.to_owned();
    let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
    tracing::debug!("node-infra-dir {:?}, {} {}", &spec.infra,  &spec.name, &infra_dir);
    let interned = intern_functions(root_dir, &infra_dir, &spec);
    functions.extend(interned);

    let version = version::current_semver(&namespace);
    let fqn = template::topology_fqn(&namespace, spec.hyphenated_names);
    let flow = Flow::new(dir, &infra_dir, &fqn, &spec);
    let mutations = make_mutations(&spec, &config);

    Topology {
        namespace: namespace.clone(),
        fqn: fqn.clone(),
        env: template::profile(),
        kind: find_kind(&spec.kind, &flow, &functions, &mutations),
        version: version,
        infra: u::gdir(&infra_dir),
        sandbox: template::sandbox(),
        dir: dir.to_string(),
        hyphenated_names: spec.hyphenated_names.to_owned(),
        nodes: nodes,
        functions: functions,
        events: make_events(&namespace, &spec, &fqn, &config),
        schedules: schedule::make_all(&infra_dir),
        routes: make_routes(&spec, &config),
        queues: make_queues(&spec, &config),
        mutations: mutations,
        channels: make_channels(&spec, &config),
        tags: tag::make(&spec.name, &infra_dir),
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
        version: version::current_semver(&namespace),
        sandbox: template::sandbox(),
        infra: u::empty(),
        dir: s!(dir),
        hyphenated_names: false,
        events: HashMap::new(),
        routes: HashMap::new(),
        flow: None,
        functions: functions,
        nodes: HashMap::new(),
        mutations: HashMap::new(),
        queues: HashMap::new(),
        channels: HashMap::new(),
        logs: LogConfig::new(),
        tags: HashMap::new(),
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

    pub fn build_functions_tree(&self) -> StringItem {
        let mut t = TreeBuilder::new(s!(self.namespace.blue()));

        for (_, f) in &self.functions {
            let vars = u::maybe_string(f.runtime.infra_spec_file.clone(), "");
            t.begin_child(s!(f.name.green()));
            t.add_empty_child(f.runtime.lang.to_str());
            t.add_empty_child(f.runtime.role.path.to_string());
            t.add_empty_child(vars);
            t.end_child();
        }

        for (_, node) in &self.nodes {
            t.begin_child(s!(&node.namespace.green()));
            for (_, f) in &node.functions {
                let vars = u::maybe_string(f.runtime.infra_spec_file.clone(), "");
                t.begin_child(s!(&f.fqn));
                t.add_empty_child(f.runtime.lang.to_str());
                t.add_empty_child(f.runtime.role.path.to_string());
                t.add_empty_child(vars);
                t.end_child();
            }
            t.end_child();
        }
        t.build()
    }

    pub fn build_nodes_tree(&self) -> StringItem {
        let mut t = TreeBuilder::new(s!(self.namespace.blue()));

        for (_, node) in &self.nodes {
            t.begin_child(s!(&node.namespace.green()));
            for (_, n) in &node.nodes {
                t.begin_child(s!(&n.namespace));
                t.add_empty_child(n.infra.to_string());
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

    pub fn graph(&self) -> Graph {
        graph::generate(self)
    }

    pub fn build_tree(&self) -> StringItem {
        let mut t = TreeBuilder::new(s!(self.namespace.blue()));
        t.begin_child(s!("functions"));
        for (_, f) in &self.functions {
            t.add_empty_child(f.fqn.clone());
        }
        t.end_child();
        t.begin_child(s!("events"));
        for (_, f) in &self.events {
            t.add_empty_child(f.name.clone());
        }
        t.end_child();
        t.build()
    }

}
