pub mod aws;
pub mod display;

mod tag;
pub mod topology;
pub mod version;

pub use aws::{
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
    mutation::Mutation,
    page,
    page::{
        BucketPolicy,
        Page,
    },
    queue::Queue,
    role::{
        Role,
        policy::Policy,
    },
    route::Route,
    schedule::Schedule,
    transducer::Transducer,
};
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
pub use display::compact::CompactTopology;
use kit as u;
use kit::*;
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

pub fn compose_dirs(dirs: Vec<String>) -> HashMap<String, Topology> {
    let mut h: HashMap<String, Topology> = HashMap::new();
    for dir in dirs {
        let abs = u::absolutize(&u::pwd(), &dir);
        let topology = compose(&abs, false);
        h.insert(topology.namespace.to_string(), topology);
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

pub fn count(topologies: &HashMap<String, Topology>) -> Vec<TopologyCount> {
    display::topology::get_count(topologies)
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

pub fn pprint(topology: &Topology, entity: Option<String>, fmt: &str) {
    let format = Format::from_str(fmt).unwrap();
    let dir = u::pwd();
    match entity {
        Some(e) => {
            let maybe_entity = Entity::from_str(&e);
            match maybe_entity {
                Ok(ent) => display::display_entity(ent, format, topology),
                Err(_) => match e.as_ref() {
                    "versions" => display::print_versions(lookup_versions(&dir), format),
                    "transducer" => u::pp_json(&topology.transducer),
                    "roles" => u::pp_json(&topology.roles),
                    _ => display::try_display(&topology, &e, format),
                },
            }
        }
        None => match format {
            Format::Tree => display::print_tree(topology),
            _ => {
                if let Some(f) = topology.current_function(&dir) {
                    u::pp_json(&f)
                } else {
                    u::pp_json(topology)
                }
            }
        },
    }
}


pub fn compact(topologies: &HashMap<String, Topology>) -> Vec<CompactTopology> {
    display::compact::build(topologies)

}
