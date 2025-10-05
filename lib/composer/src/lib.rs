
mod display;
mod aws;

mod tag;
pub mod topology;
pub mod version;


use compiler::{
    entity::Entity,
    spec::{
        TopologySpec,
        function::LangRuntime,
    },
};
use configurator::Config;
use display::Format;
pub use display::topology::TopologyCount;
use kit as u;
use kit::*;
pub use aws::function::{
    Function,
    build::Build,
    layer::Layer,
    runtime::Runtime,
};
pub use aws::event::{Event, Target};
pub use aws::channel::Channel;
pub use aws::mutation::Mutation;
pub use aws::page::{Page, BucketPolicy};
pub use aws::queue::Queue;
pub use aws::role::Role;
pub use aws::route::Route;
pub use aws::schedule::Schedule;
pub use aws::flow::Flow;
pub use aws::role::policy::Policy;

pub use aws::function;
pub use aws::page;

use std::{
    collections::HashMap,
    str::FromStr,
};
pub use topology::Topology;
use walkdir::WalkDir;

pub fn is_root_dir(dir: &str) -> bool {
    let f = format!("{}/topology.yml", dir);
    topology::is_root_topology(&f)
}

pub fn config(dir: &str) -> Config {
    let t = Topology::new(dir, false, true);
    t.config
}

fn should_recurse(given: bool, maybe_bool: Option<bool>) -> bool {
    match maybe_bool {
        Some(b) => b,
        None => given,
    }
}

pub fn compose(dir: &str, recursive: bool) -> Topology {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    let recurse = should_recurse(recursive, spec.recursive);
    Topology::new(dir, recurse, false)
}

pub fn lookup_versions(dir: &str) -> HashMap<String, String> {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    let given_root_dirs = match &spec.nodes.dirs {
        Some(dirs) => dirs,
        None => &list_dirs(dir),
    };
    let mut h: HashMap<String, String> = HashMap::new();
    for d in given_root_dirs {
        let f = format!("{}/{}/topology.yml", dir, &d);
        let spec = TopologySpec::new(&f);
        if &spec.name != "tc" {
            let version = version::current_semver(&spec.name);
            h.insert(spec.name, version);
        }
    }
    h
}

pub fn compose_root(dir: &str, recursive: bool) -> HashMap<String, Topology> {
    let f = format!("{}/topology.yml", dir);
    if u::file_exists(&f) {
        let spec = TopologySpec::new(&f);
        let given_root_dirs = match &spec.nodes.dirs {
            Some(dirs) => dirs,
            None => &list_dirs(dir),
        };
        let mut h: HashMap<String, Topology> = HashMap::new();
        if given_root_dirs.is_empty() {
            let topology = compose(&u::pwd(), false);
            h.insert(topology.namespace.clone(), topology);
        } else {
            for d in given_root_dirs {
                tracing::debug!("Given root: {}", &d);
                let dir = u::absolutize(dir, &d);
                let t = compose(&dir, recursive);
                h.insert(t.namespace.to_string(), t);
            }
        }
        tracing::debug!("Compilation completed");
        h
    } else {
        let dirs = u::list_dirs(dir);
        let mut h: HashMap<String, Topology> = HashMap::new();
        for d in dirs {
            let f = format!("{}/topology.yml", d);
            if u::file_exists(&f) {
                let topology = compose(&d, recursive);
                h.insert(topology.namespace.clone(), topology);
            }
        }
        h
    }
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

// deprecated
pub fn find_layers() -> Vec<Layer> {
    let dir = u::pwd();
    if topology::is_compilable(&dir) {
        let topology = compose(&dir, true);
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

// display

pub fn print_topologies(format: &str, topologies: HashMap<String, Topology>) {
    match format {
        "table" => display::topology::print_stats(topologies),
        "json" => display::topology::print_stats_json(topologies),
        "tree" => {
            println!("")
        }
        _ => (),
    }
}

pub fn display_root() {
    let topologies = list_topologies();
    display::topology::print_stats(topologies)
}

pub fn display_topology(dir: &str, format: &str, recursive: bool) {
    let topology = compose(&dir, recursive);
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

    match e {
        "." => {
            let topology = compose(&dir, recursive);
            if let Some(f) = topology.current_function(dir) {
                u::pp_json(&f)
            }
        }
        "versions" => {
            let versions = lookup_versions(dir);
            display::topology::print_versions(versions, f);
        }

        "stats" => {
            let topologies = compose_root(dir, true);
            display::topology::print_stats(topologies)
        }

        "roles" => {
            let topology = compose(&dir, recursive);
            u::pp_json(&topology.roles);
        }
        _ => {
            let topology = compose(&dir, recursive);
            display::try_display(&topology, e, format)
        }
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
    version::current_semver(&namespace)
}

pub fn current_function(dir: &str) -> Option<Function> {
    let topology = Topology::new(dir, false, true);
    topology.current_function(dir)
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
    display::topology::count_str(topology)
}

pub fn entities_of(topology: &Topology) -> Vec<Entity> {
    let Topology {
        routes,
        events,
        channels,
        queues,
        functions,
        pages,
        mutations,
        flow,
        ..
    } = topology;
    let mut xs: Vec<Entity> = vec![];

    if functions.len() > 0 {
        xs.push(Entity::Function)
    }
    if routes.len() > 0 {
        xs.push(Entity::Route)
    }
    if events.len() > 0 {
        xs.push(Entity::Event)
    }
    if pages.len() > 0 {
        xs.push(Entity::Page)
    }
    if channels.len() > 0 {
        xs.push(Entity::Channel)
    }
    if queues.len() > 0 {
        xs.push(Entity::Queue)
    }
    if let Some(_f) = flow {
        xs.push(Entity::State);
    }
    if mutations.len() > 0 {
        if let Some(m) = mutations.get("default") {
            if m.resolvers.len() > 0 {
                xs.push(Entity::Mutation)
            }
        }
    }

    xs
}
