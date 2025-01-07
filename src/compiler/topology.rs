use crate::compiler::flow;
use crate::compiler::layer;
use crate::compiler::layer::Layer;
use colored::Colorize;
use ptree::builder::TreeBuilder;
use ptree::item::StringItem;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

use super::{BasicSpec, Events, FunctionSpec, Queue, Route, Schedule, Spec};
use crate::compiler::config::Config;
use crate::compiler::mutation;
use crate::compiler::mutation::Mutations;
use crate::compiler::schedule;
use kit as u;
use kit::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Filter {
    pub metadata: HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Topology {
    pub name: String,
    pub kind: String,
    pub mode: Option<String>,
    pub nodes: Vec<Topology>,
    pub infra: String,
    pub role: Option<String>,
    pub dir: String,
    pub version: Option<String>,
    pub events: Option<Events>,
    pub hyphenated_names: bool,
    pub routes: Option<HashMap<String, Route>>,
    pub functions: HashMap<String, FunctionSpec>,
    pub mutations: Option<Mutations>,
    pub schedules: HashMap<String, Schedule>,
    pub queues: HashMap<String, Queue>,
    pub flow: Option<Value>,
    pub config: Config,
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

fn infra_dir(given_infra_dir: Option<String>, namespace: &str) -> String {
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

fn intern_functions(root_dir: &str, infra_dir: &str, spec: &Spec) -> HashMap<String, FunctionSpec> {
    let shared = spec.clone().functions.shared;
    let namespace = &spec.name;

    let mut functions: HashMap<String, FunctionSpec> = HashMap::new();
    for d in shared {
        let abs_dir = u::absolute_dir(root_dir, &d);
        if u::is_dir(&abs_dir) {
            let function = FunctionSpec::new(&abs_dir, infra_dir, &namespace, spec.fmt());
            match function {
                Some(f) => functions.insert(abs_dir, f),
                None => None,
            };
        }
    }
    functions
}

fn should_ignore_node(root_dir: &str, ignore_nodes: Vec<String>, topology_dir: &str) -> bool {
    for node in ignore_nodes {
        let abs_path = format!("{root_dir}/{node}");
        if &abs_path == topology_dir {
            return true;
        }
        if topology_dir.starts_with(&abs_path) {
            return true;
        }
    }
    return false;
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

fn discover_functions(dir: &str, infra_dir: &str, spec: &Spec) -> HashMap<String, FunctionSpec> {
    let mut functions: HashMap<String, FunctionSpec> = HashMap::new();
    let dirs = function_dirs(dir);

    for d in dirs {
        if u::is_dir(&d) && !&d.starts_with(".") {
            let function = FunctionSpec::new(&d, infra_dir, &spec.name, spec.fmt());
            match function {
                Some(f) => functions.insert(d, f),
                None => None,
            };
        }
    }
    functions
}

fn current_function(dir: &str, infra_dir: &str, spec: &Spec) -> HashMap<String, FunctionSpec> {
    let mut functions: HashMap<String, FunctionSpec> = HashMap::new();
    if u::is_dir(dir) && !dir.starts_with(".") {
        let function = FunctionSpec::new(dir, infra_dir, &spec.name, spec.fmt());
        match function {
            Some(f) => functions.insert(dir.to_string(), f),
            None => None,
        };
    }
    functions
}

fn role_of(infra_dir: &str) -> Option<String> {
    let path = format!("{}/roles/sfn.json", infra_dir);
    if u::file_exists(&path) {
        Some(path)
    } else {
        None
    }
}

fn discover_leaf_nodes(root_dir: &str, infra_dir: &str, dir: &str, spec: &Spec) -> Vec<Topology> {
    let ignore_nodes = &spec.nodes.ignore;

    let mut nodes: Vec<Topology> = vec![];
    if is_topology_dir(dir) {
        if !should_ignore_node(root_dir, ignore_nodes.clone(), dir) {
            let f = format!("{}/topology.yml", dir);
            let spec = Spec::new(&f);
            let mut functions = discover_functions(dir, infra_dir, &spec);
            let interned = intern_functions(dir, infra_dir, &spec);
            functions.extend(interned);
            let node = make(root_dir, dir, &spec, functions, vec![]);
            nodes.push(node);
        }
    }
    nodes
}

pub fn discover_nodes(root_dir: &str, infra_dir: &str, spec: &Spec) -> Vec<Topology> {
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
                let spec = Spec::new(&f);
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

fn make(
    root_dir: &str,
    dir: &str,
    spec: &Spec,
    functions: HashMap<String, FunctionSpec>,
    nodes: Vec<Topology>,
) -> Topology {
    let mut functions = functions;
    let namespace = spec.name.to_owned();
    let infrastructure_dir = infra_dir(spec.infra.to_owned(), &spec.name);
    let interned = intern_functions(root_dir, &infrastructure_dir, &spec);
    functions.extend(interned);

    Topology {
        name: namespace,
        kind: spec.kind.to_owned(),
        version: spec.version.to_owned(),
        infra: infrastructure_dir.to_owned(),
        role: role_of(&infrastructure_dir),
        nodes: nodes,
        dir: s!(dir),
        mode: spec.mode.to_owned(),
        hyphenated_names: spec.hyphenated_names.to_owned(),
        events: spec.events.to_owned(),
        schedules: schedule::make(&infrastructure_dir),
        routes: spec.routes.to_owned(),
        functions: functions,
        queues: spec.queues.to_owned(),
        mutations: mutation::make(&spec.name, spec.mutations.to_owned()),
        flow: match &spec.flow {
            Some(f) => Some(flow::read(dir, f.clone())),
            None => spec.states.to_owned(),
        },
        config: Config::new(),
    }
}

fn make_standalone(dir: &str) -> Topology {
    let function = FunctionSpec::new(dir, dir, "", "");
    let functions = FunctionSpec::to_map(function.clone());
    let f = function.unwrap();

    Topology {
        name: f.name.to_owned(),
        kind: s!("function"),
        version: None,
        infra: u::empty(),
        role: None,
        dir: s!(dir),
        mode: None,
        hyphenated_names: false,
        events: None,
        routes: None,
        flow: None,
        functions: functions,
        nodes: vec![],
        mutations: None,
        queues: HashMap::new(),
        schedules: HashMap::new(),
        config: Config::new(),
    }
}

pub fn basic_spec(dir: &str) -> BasicSpec {
    if is_topology_dir(dir) {
        let f = format!("{}/topology.yml", dir);
        BasicSpec::new(&f)
    } else {
        BasicSpec {
            name: String::from("default"),
            hyphenated_names: false,
        }
    }
}

fn make_relative(dir: &str) -> Topology {
    let f = match parent_topology_file(dir) {
        Some(file) => file,
        None => format!("../topology.yml"),
    };

    let spec = Spec::new(&f);
    let namespace = &spec.name;
    let infra_dir = infra_dir(spec.infra.to_owned(), &spec.name);
    let fspec = FunctionSpec::new(dir, &infra_dir, namespace, &spec.fmt());
    let functions = FunctionSpec::to_map(fspec);
    let nodes = vec![];
    make(dir, dir, &spec, functions, nodes)
}

pub fn is_compilable(dir: &str) -> bool {
    is_standalone_function_dir(dir) || is_relative_topology_dir(dir) || is_topology_dir(dir)
}

impl Topology {
    pub fn new(dir: &str, recursive: bool) -> Topology {
        if is_singular_function_dir() {
            let f = format!("{}/topology.yml", dir);
            let spec = Spec::new(&f);
            let infra_dir = infra_dir(spec.infra.to_owned(), &spec.name);
            let functions = current_function(dir, &infra_dir, &spec);
            make(dir, dir, &spec, functions, vec![])
        } else if is_topology_dir(dir) {
            let f = format!("{}/topology.yml", dir);
            let spec = Spec::new(&f);
            let infra_dir = infra_dir(spec.infra.to_owned(), &spec.name);
            let functions = discover_functions(dir, &infra_dir, &spec);
            let nodes;
            if recursive {
                nodes = discover_nodes(dir, &infra_dir, &spec);
            } else {
                nodes = vec![];
            }
            make(dir, dir, &spec, functions, nodes)
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

    pub fn functions(&self) -> HashMap<String, FunctionSpec> {
        let mut fns: HashMap<String, FunctionSpec> = self.clone().functions;
        for node in &self.nodes {
            fns.extend(node.clone().functions);
        }
        fns.clone()
    }

    pub fn build_tree(&self) -> StringItem {
        let mut t = TreeBuilder::new(s!(self.name.blue()));

        for (_, f) in &self.functions {
            t.begin_child(s!(f.name.green()));
            t.add_empty_child(f.runtime.lang.to_owned());
            t.add_empty_child(f.runtime.layers.join(","));
            t.add_empty_child(u::value_to_string(f.assets.get("DEPS_PATH")));
            t.end_child();
        }

        for node in &self.nodes {
            t.begin_child(s!(&node.name.green()));
            for (_, f) in &node.functions {
                t.begin_child(s!(&f.fqn));
                t.add_empty_child(f.runtime.lang.to_owned());
                t.add_empty_child(f.runtime.layers.join(","));
                t.add_empty_child(u::value_to_string(f.assets.get("DEPS_PATH")));
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
}
