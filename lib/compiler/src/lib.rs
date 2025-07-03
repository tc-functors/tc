mod display;
mod entity;
mod parser;
pub mod spec;
pub mod topology;

use display::Format;
pub use display::topology::TopologyCount;
pub use entity::Entity;
use kit as u;
use kit::*;
pub use spec::{
    TopologyKind,
    TopologySpec,
    config::ConfigSpec,
    function::{
        BuildKind,
        Lang,
        LangRuntime,
    },
    infra::InfraSpec,
};
use std::{
    collections::HashMap,
    str::FromStr,
};
pub use topology::{
    Topology,
    channel::Channel,
    event::{
        Event,
        Target,
    },
    flow::Flow,
    function,
    function::{
        Function,
        build::Build,
        layer::Layer,
        runtime::Runtime,
    },
    mutation,
    mutation::Mutation,
    page::Page,
    queue::Queue,
    role::Role,
    route,
    route::Route,
    schedule::Schedule,
};
use walkdir::WalkDir;

pub fn is_root_dir(dir: &str) -> bool {
    let f = format!("{}/topology.yml", dir);
    topology::is_root_topology(&f)
}

pub fn config(dir: &str) -> ConfigSpec {
    let t = Topology::new(dir, false, true);
    t.config
}

pub fn compile(dir: &str, recursive: bool) -> Topology {
    if u::path_exists(dir, "topology.lisp") {
        let f = format!("{}/topology.lisp", dir);
        let data = u::slurp(&f);
        let program = format!("{{ {data} }}");
        parser::lisp::load(program);
        Topology::new(dir, recursive, false)
    } else {
        Topology::new(dir, recursive, false)
    }
}

pub fn compile_root(dir: &str, recursive: bool) -> HashMap<String, Topology> {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    let given_root_dirs = match &spec.nodes.dirs {
        Some(dirs) => dirs,
        None => &list_dirs(dir),
    };
    let mut h: HashMap<String, Topology> = HashMap::new();
    if given_root_dirs.is_empty() {
        let topology = compile(&u::pwd(), false);
        h.insert(topology.namespace.clone(), topology);
    } else {
        for d in given_root_dirs {
            tracing::debug!("Given root: {}", &d);
            let dir = u::absolutize(&u::pwd(), &d);
            let t = compile(&dir, recursive);

            h.insert(t.namespace.to_string(), t);
        }
    }
    tracing::debug!("Compilation completed");
    h
}

pub fn root_namespaces(dir: &str) -> HashMap<String, String> {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    let given_root_dirs = match &spec.nodes.dirs {
        Some(dirs) => dirs,
        None => &list_dirs(dir),
    };
    let mut h: HashMap<String, String> = HashMap::new();
    for d in given_root_dirs {
        let name = topology_name(d);
        h.insert(d.to_string(), name);
    }
    h
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

pub fn is_topology_dir(dir: &str) -> bool {
    topology::is_topology_dir(dir)
}

pub fn display_root() {
    let topologies = list_topologies();
    display::topology::print_topologies(topologies)
}

pub fn display_topology(dir: &str, format: &str, recursive: bool) {
    let topology = compile(&dir, recursive);
    match format {
        "tree" => {
            let tree = display::topology::build_tree(&topology);
            kit::print_tree(tree);
        }
        _ => (),
    }
}

pub fn display_entity(dir: &str, e: &str, f: &str, recursive: bool) {
    let format = Format::from_str(f).unwrap();

    let topology = compile(&dir, recursive);

    if e == "." {
        if let Some(f) = topology.current_function(dir) {
            u::pp_json(&f)
        }
    } else {
        display::try_display(&topology, e, format);
    }
}

pub fn pprint(topology: &Topology, entity: Option<Entity>) {
    let fmt = Format::JSON;
    match entity {
        Some(e) => display::display_entity(e, fmt, topology),
        None => u::pp_json(topology),
    }
}

pub fn topology_name(dir: &str) -> String {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    spec.name
}

pub fn topology_version(namespace: &str) -> String {
    topology::version::current_semver(&namespace)
}

pub fn current_function(dir: &str) -> Option<Function> {
    let topology = Topology::new(dir, false, true);
    topology.current_function(dir)
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

pub fn count_of(topology: &Topology) -> String {
    let Topology {
        functions,
        mutations,
        events,
        queues,
        routes,
        pages,
        ..
    } = topology;

    let mut f: usize = functions.len();
    let mut m: usize = match mutations.get("default") {
        Some(mx) => mx.resolvers.len(),
        _ => 0,
    };
    let mut e: usize = events.len();
    let mut q: usize = queues.len();
    let mut r: usize = routes.len();
    let mut p: usize = pages.len();

    let nodes = &topology.nodes;

    for (_, node) in nodes {
        let Topology {
            functions,
            mutations,
            events,
            queues,
            routes,
            pages,
            ..
        } = node;
        f = f + functions.len();
        m = m + match mutations.get("default") {
            Some(mx) => mx.resolvers.len(),
            _ => 0,
        };
        e = e + events.len();
        q = q + queues.len();
        r = r + routes.len();
        p = p + pages.len();
    }

    let msg = format!(
        "nodes: {}, functions: {}, mutations: {}, events: {}, routes: {}, queues: {}, pages: {}",
        nodes.len() + 1,
        f,
        m,
        e,
        r,
        q,
        p
    );
    msg
}
