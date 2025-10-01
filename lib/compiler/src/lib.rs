pub mod entity;
mod lisp;
mod printer;
pub mod spec;
mod yaml;
mod walker;

pub use entity::Entity;
use kit as u;
pub use spec::{
    TopologyKind,
    TopologySpec,
    TopologyMetadata,
    function,
    function::{
        build::{BuildKind, BuildSpec},
        FunctionSpec,
        runtime::Lang,
        runtime::LangRuntime,
        infra::InfraSpec,
    },
    test::TestSpec,
    role::RoleSpec,
    mutation::MutationSpec,
    route::RouteSpec,
    schedule::ScheduleSpec,
    page::PageSpec
};
use std::{
    collections::HashMap,
    path::Path,
    str::FromStr
};
use printer::Format;

fn should_recurse(given: bool, maybe_bool: Option<bool>) -> bool {
    match maybe_bool {
        Some(b) => b,
        None => given,
    }
}

pub fn compile(dir: &str, recursive: bool) -> TopologySpec {
    let yaml_file = format!("{}/topology.yml", dir);
    let lisp_file = format!("{}/topology.lisp", dir);
    let spec = if u::file_exists(&yaml_file) {
        TopologySpec::new(&yaml_file)
    } else if u::file_exists(&lisp_file) {
        let data = u::slurp(&lisp_file);
        lisp::load(data);
        TopologySpec::new(&yaml_file)
    } else {
        walker::make_standalone(dir)
    };
    let recurse = should_recurse(recursive, spec.recursive);
    spec.walk(recurse)
}

pub fn compile_root(dir: &str, recursive: bool) -> HashMap<String, TopologySpec> {
    let f = format!("{}/topology.yml", dir);
    if u::file_exists(&f) {
        let spec = TopologySpec::new(&f);
        let given_root_dirs = match &spec.nodes.dirs {
            Some(dirs) => dirs,
            None => &u::list_dirs(dir),
        };
        let mut h: HashMap<String, TopologySpec> = HashMap::new();
        if given_root_dirs.is_empty() {
            let tspec = compile(&u::pwd(), recursive);
            h.insert(tspec.name.clone(), tspec);
        } else {
            for d in given_root_dirs {
                tracing::debug!("Given root: {}", &d);
                let dir = u::absolutize(dir, &d);
                let t = compile(&dir, recursive);
                h.insert(t.name.to_string(), t);
            }
        }
        tracing::debug!("Compilation completed");
        h
    } else {
        let dirs = u::list_dirs(dir);
        let mut h: HashMap<String, TopologySpec> = HashMap::new();
        for d in dirs {
            let f = format!("{}/topology.yml", d);
            if u::file_exists(&f) {
                let tspec = compile(&d, recursive);
                h.insert(tspec.name.clone(), tspec);
            }
        }
        h
    }
}

pub async fn repl() {
    let _ = lisp::repl().await;
}

pub fn load(data: &str) {
    lisp::load(data.to_string());
}

pub fn guess_runtime(dir: &str) -> LangRuntime {
    function::infer_lang(dir)
}

pub fn is_topology_dir(dir: &str) -> bool {
    let topology_file = format!("{}/topology.yml", dir);
    Path::new(&topology_file).exists()
}

pub fn root_namespaces(dir: &str) -> HashMap<String, String> {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    let given_root_dirs = match &spec.nodes.dirs {
        Some(dirs) => dirs,
        None => &u::list_dirs(dir),
    };
    let mut h: HashMap<String, String> = HashMap::new();
    for d in given_root_dirs {
        let name = namespace_of(d);
        h.insert(d.to_string(), name);
    }
    h
}

pub fn namespace_of(dir: &str) -> String {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    spec.name
}

pub fn version_of(namespace: &str) -> String {
    u::current_semver(&namespace)
}

pub fn lookup_versions(dir: &str) -> HashMap<String, String> {
    let f = format!("{}/topology.yml", dir);
    let spec = TopologySpec::new(&f);
    let given_root_dirs = match &spec.nodes.dirs {
        Some(dirs) => dirs,
        None => &u::list_dirs(dir),
    };
    let mut h: HashMap<String, String> = HashMap::new();
    for d in given_root_dirs {
        let f = format!("{}/{}/topology.yml", dir, &d);
        let spec = TopologySpec::new(&f);
        if &spec.name != "tc" {
            let version = u::current_semver(&spec.name);
            h.insert(spec.name, version);
        }
    }
    h
}

pub fn pprint(spec: &TopologySpec, format: &str) {
    let fmt = Format::from_str(format).unwrap();
    match fmt {
        Format::Tree => printer::print_tree(spec),
        Format::JSON => u::pp_json(spec),
        Format::Bincode => spec.to_bincode(),
        _ => u::pp_json(spec),
    }
}

pub fn pprint_component(dir: &str, component: &str, format: &str) {
    match component {
        "versions" => {
            let versions = lookup_versions(dir);
            printer::print_versions(versions, format);
        },
        "counts" | "count" => {
            let dir =  u::root();
            let topologies = compile_root(&dir, true);
            printer::print_count(topologies);
        },
        _ => {
            let fmt = Format::from_str(format).unwrap();
            let entity = Entity::from_str(component).unwrap();
            let topology = compile(dir, false);
            printer::print_entity(&topology, entity, fmt);
        }
    }
}

pub fn print_specs(specs: HashMap<String, TopologySpec>, _fmt: &str) {
    printer::print_count(specs)
}

pub fn is_root_dir(dir: &str) -> bool {
    let f = format!("{}/topology.yml", dir);
    is_root_topology(&f)
}

pub fn is_root_topology(spec_file: &str) -> bool {
    let spec = TopologySpec::new(spec_file);
    if let Some(given_root_dirs) = &spec.nodes.dirs {
        !given_root_dirs.is_empty()
    } else {
        spec.nodes.root.is_some()
    }
}

pub fn print_root(dir: &str) {
    let specs = compile_root(dir, true);
    print_specs(specs, "")
}
