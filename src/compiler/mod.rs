use std::collections::HashMap;
use std::path::Path;

pub mod spec;
pub mod topology;

pub mod deps;

pub mod config;
pub mod flow;
pub mod function;
pub mod layer;
pub mod mutation;
pub mod schedule;

use colored::Colorize;
pub use config::Config;
pub use function::{FunctionSpec, RuntimeSpec};
pub use layer::Layer;
pub use mutation::Mutations;
pub use schedule::Schedule;
use serde_json::Value;
pub use spec::{BasicSpec, Consumes, Events, MutationSpec, Queue, ResolverSpec, Route, Spec};
pub use topology::Topology;
use walkdir::WalkDir;

use kit as u;
use kit::*;

pub fn compile(dir: &str, recursive: bool) -> Topology {
    Topology::new(dir, recursive)
}

pub fn just_functions() -> HashMap<String, FunctionSpec> {
    let mut functions: HashMap<String, FunctionSpec> = HashMap::new();
    let dir = u::pwd();

    for entry in WalkDir::new(dir.clone())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if topology::is_topology_dir(&p) {
            let topology = Topology::new(&p, false);
            let fns = topology.functions();
            functions.extend(fns);
        }
    }
    functions
}

pub fn find_layers() -> Vec<Layer> {
    let dir = u::pwd();
    if topology::is_compilable(&dir) {
        let topology = compile(&dir, true);
        topology.layers()
    } else {
        layer::discover()
    }
}

pub fn find_layer_names() -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    let layers = find_layers();
    for layer in layers {
        if layer.target != "efs" {
            xs.push(layer.name);
        }
    }
    u::uniq(xs)
}

pub fn guess_lang(dir: &str) -> String {
    function::infer_lang(dir).to_string()
}

pub fn is_topology_dir(dir: &str) -> bool {
    topology::is_topology_dir(dir)
}

pub fn show_component(component: &str, format: &str) -> String {
    let dir = u::pwd();
    match component {
        "layers" => {
            let layers = find_layers();
            u::pretty_json(layers)
        }
        "states" => {
            let topology = compile(&dir, false);
            match topology.flow {
                Some(f) => u::pretty_json(&f),
                None => u::empty(),
            }
        }
        "routes" => {
            let topology = compile(&dir, false);
            u::pretty_json(&topology.routes)
        }
        "roles" | "vars" => {
            let topology = compile(&dir, true);
            let functions = topology.functions();
            for (_dir, f) in functions {
                println!("{}", &f.fqn.blue());
                println!("  role: - {}", &f.role.path);
                if let Some(v) = &f.vars_file {
                    println!("  var:  - {}", &v);
                }
            }
            u::empty()
        }
        "events" => {
            let topology = compile(&dir, false);
            u::pretty_json(&topology.events)
        }
        "schedules" => {
            let topology = compile(&dir, false);
            u::pretty_json(&topology.schedules)
        }
        "functions" => {
            let topology = compile(&dir, true);
            match format {
                "tree" => {
                    let tree = topology.build_tree();
                    kit::print_tree(tree);
                    u::empty()
                }
                "json" => u::pretty_json(&topology.functions),
                _ => u::pretty_json(&topology.functions),
            }
        }
        "mutations" => {
            let topology = compile(&dir, false);
            if format == "graphql" {
                mutation::print_graphql(&topology.mutations.unwrap().types);
                u::empty()
            } else {
                u::pretty_json(&topology.mutations)
            }
        }

        "topologies" => {
            let topologies = list_topologies();
            for (dir, basic_spec) in topologies {
                let BasicSpec { name, .. } = basic_spec;
                println!("{} - {}", &name, u::second(&dir, "/services/"));
            }
            u::empty()
        }

        "dirs" => {
            let topologies = list_topology_dirs();
            for (name, dir) in topologies {
                println!("{} - {}", &name, &dir);
            }
            u::empty()
        },

        "config" => {
            let cfg = Config::new();
            println!("{:?}", &cfg);
            u::empty()
        }

        _ => {
            let topology = compile(&dir, true);
            if u::file_exists(&component) {
                let functions = topology.functions;
                let fn_dir = format!("{}/{}", &dir, component);
                let f = functions.get(&fn_dir).unwrap();
                u::pretty_json(f)
            } else {
                u::empty()
            }
        }
    }
}

pub fn topology_name(dir: &str) -> String {
    let f = format!("{}/topology.yml", dir);
    let spec = Spec::new(&f);
    spec.name
}

pub fn topology_mode(dir: &str) -> &str {
    let f = format!("{}/topology.yml", dir);
    let spec = Spec::new(&f);
    match spec.mode {
        Some(m) => match m.as_ref() {
            "Standard" => "async",
            "Expresss" => "sync",
            _ => "sync",
        },
        None => "async",
    }
}

pub fn current_function(dir: &str) -> Option<FunctionSpec> {
    let topology = Topology::new(dir, false);
    topology
        .functions
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .first()
        .cloned()
}

pub fn kind_of() -> String {
    let dir = &u::pwd();
    if topology::is_topology_dir(dir) {
        s!("step-function")
    } else if u::file_exists("function.json") {
        s!("function")
    } else {
        s!("event")
    }
}

pub fn find_assets(dir: &str) -> HashMap<String, Value> {
    let topology = Topology::new(dir, false);
    deps::find_assets(dir, &topology)
}

pub fn determine_target(dir: &str) -> String {
    if is_topology_dir(&u::pwd()) || u::path_exists(dir, "function.json") {
        let assets = find_assets(dir);
        let is_efs = assets.get("MODEL_PATH").is_some();
        if !is_efs {
            s!("layer")
        } else {
            s!("efs")
        }
    } else {
        s!("layer")
    }
}

pub fn list_topologies() -> HashMap<String, BasicSpec> {
    let mut names: Vec<String> = vec![];
    let mut topologies: HashMap<String, BasicSpec> = HashMap::new();
    for entry in WalkDir::new(u::pwd())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if is_topology_dir(&p) {
            let spec = topology::basic_spec(&p);
            if !names.contains(&spec.name.to_string()) {
                names.push(spec.name.to_string());
                topologies.insert(p.to_string(), spec);
            }
        }
    }
    topologies
}

fn is_ci_dir(dir: &str) -> bool {
    //FIXME: handle hidden dirs
    let ci_dir = format!("{}/.circleci", dir);
    Path::new(&ci_dir).exists()
}

pub fn list_topology_dirs() -> HashMap<String, String> {
    let mut topologies: HashMap<String, String> = HashMap::new();
    for entry in WalkDir::new(u::pwd())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if is_topology_dir(&p) && is_ci_dir(&p) {
            let spec = topology::basic_spec(&p);
            topologies.insert(spec.name.to_string(), p.to_string());
        }
    }
    topologies
}

pub fn _get_config() -> Config {
    Config::new()
}
