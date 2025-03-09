use std::collections::HashMap;
use std::path::Path;

pub mod spec;
pub mod topology;
pub mod function;
mod flow;
mod mutation;
mod schedule;
pub mod event;
pub mod route;
mod queue;
mod version;
mod template;
mod role;
mod log;

use colored::Colorize;
use walkdir::WalkDir;

pub use function::layer::Layer;
pub use mutation::{Mutation, Resolver};
pub use schedule::Schedule;
pub use topology::Topology;
pub use function::{Function, Build, Runtime};
pub use event::{Event, TargetKind, Target};
pub use queue::Queue;
pub use route::Route;
pub use flow::Flow;
pub use role::{Role, RoleKind};
pub use log::LogConfig;

pub use spec::{TopologySpec, LangRuntime, Lang, RuntimeInfraSpec, BuildKind, TopologyKind};

use kit as u;
use kit::*;

pub fn compile(dir: &str, recursive: bool) -> Topology {
    Topology::new(dir, recursive, false)
}

pub fn just_functions() -> HashMap<String, Function> {
    let mut functions: HashMap<String, Function> = HashMap::new();
    let dir = u::pwd();

    for entry in WalkDir::new(dir.clone())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if topology::is_topology_dir(&p) {
            let topology = Topology::new(&p, false, false);
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
        function::layer::discover()
    }
}

pub fn find_buildables(dir: &str, recursive: bool) -> Vec<Build> {
    let mut xs: Vec<Build> = vec![];
    let topology = Topology::new(dir, recursive, false);
    let fns = topology.functions;
    for (_, f) in fns {
        xs.push(f.build)
    }
    xs
}


pub fn find_layer_names() -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    let layers = find_layers();
    for layer in layers {
        xs.push(layer.name)
    }
    u::uniq(xs)
}

pub fn guess_runtime(dir: &str) -> LangRuntime {
    function::runtime::infer_lang(dir)
}

pub fn guess_build_runtime(dir: &str, kind: BuildKind) -> LangRuntime {
    match kind {
        BuildKind::Code => function::runtime::infer_lang(dir),
        BuildKind::Layer | BuildKind::Library => {
            let dir = u::nths(list_dir(dir), 0);
            function::runtime::infer_lang(&dir)
        },
        _ => function::runtime::infer_lang(dir),
    }

}

pub fn guess_lang(dir: &str) -> Lang {
    function::runtime::infer_lang(dir).to_lang()
}

pub fn is_topology_dir(dir: &str) -> bool {
    topology::is_topology_dir(dir)
}

pub fn show_component(component: &str, format: &str, recursive: bool) -> String {
    let dir = u::pwd();
    match component {
        "layers" => {
            let layers = find_layers();
            u::pretty_json(layers)
        }
        "flow" => {
            let topology = compile(&dir, recursive);
            match topology.flow {
                Some(f) => u::pretty_json(&f),
                None => u::empty(),
            }
        }
        "routes" => {
            let topology = compile(&dir, recursive);
            u::pretty_json(&topology.routes)
        }
        "runtime" => {
            let topology = compile(&dir, recursive);
            let functions = topology.functions();
            for (_dir, f) in functions {
                println!("{}", u::pretty_json(f.runtime));
            }
            u::empty()
        }
        "vars" => {
            let topology = compile(&dir, recursive);
            let functions = topology.functions();
            for (_dir, f) in functions {
                println!("{}", u::pretty_json(f.runtime));
            }
            u::empty()
        }
        "events" => {
            if recursive {
                let topologies = list_topologies();
                let mut h: HashMap<String, Event> = HashMap::new();
                for (_dir, t) in topologies {
                    let Topology { events, .. } = t;
                    h.extend(events);
                }
                println!("{}", u::pretty_json(h));
                u::empty()
            } else {
                let topology = compile(&dir, false);
                u::pretty_json(&topology.events)
            }
        }
        "schedules" => {
            let topology = compile(&dir, recursive);
            u::pretty_json(&topology.schedules)
        }
        "functions" => {
            let topology = compile(&dir, recursive);
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
            let topology = compile(&dir, recursive);
            if format == "graphql" {
                mutation::print_graphql(&topology.mutations.values().into_iter().nth(0).unwrap().types);
                u::empty()
            } else {
                u::pretty_json(&topology.mutations)
            }
        }

        "topologies" => {
            let topologies = list_topologies();
            for (dir, basic_spec) in topologies {
                let Topology { namespace, .. } = basic_spec;
                println!("{} - {}", &namespace, u::second(&dir, "/services/"));
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

        _ => {
            let topology = compile(&dir, recursive);
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
    let spec = TopologySpec::new(&f);
    spec.name
}

pub fn current_function(dir: &str) -> Option<Function> {
    let topology = Topology::new(dir, false, true);
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

pub fn list_topologies() -> HashMap<String, Topology> {
    let mut names: Vec<String> = vec![];
    let mut topologies: HashMap<String, Topology> = HashMap::new();
    for entry in WalkDir::new(u::pwd())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if is_topology_dir(&p) {
            let spec = Topology::new(&p, true, true);
            if !names.contains(&spec.namespace.to_string()) {
                names.push(spec.namespace.to_string());
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
            let spec = Topology::new(&p, false, true);
            topologies.insert(spec.namespace.to_string(), p.to_string());
        }
    }
    topologies
}

pub fn count_of(topology: &Topology) {

    let Topology { functions, mutations, events, queues, routes, .. } = topology;

    let mut f: usize = functions.len();
    let mut m: usize = match mutations.get("default") {
        Some(mx) => mx.resolvers.len(),
        _ => 0
    };
    let mut e: usize = events.len();
    let mut q: usize = queues.len();
    let mut r: usize = routes.len();

    let nodes = &topology.nodes;

    for node in nodes {
        let Topology { functions, mutations, events, queues, routes, .. } = node;
        f = f  + functions.len();
        m = m + match mutations.get("default") {
            Some(mx) => mx.resolvers.len(),
            _ => 0
        };
        e = e + events.len();
        q = q + queues.len();
        r = r + routes.len();
    }

    let msg = format!("{} functions, {} mutations, {} events, {} routes, {} queues",
                      f, m, e, r, q);
    println!("Compiling {} ", msg.cyan());
}
