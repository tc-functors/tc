pub mod entity;
mod lisp;
mod printer;
pub mod spec;
mod yaml;

pub use entity::Entity;
use kit as u;
pub use spec::{
    TopologyKind,
    TopologySpec,
    function,
    function::{
        BuildKind,
        FunctionSpec,
        Lang,
        LangRuntime,
    },
    infra::InfraSpec,
};
use std::{
    collections::HashMap,
    path::Path,
};

pub fn compile(dir: &str) -> TopologySpec {
    let yaml_file = format!("{}/topology.yml", dir);
    let lisp_file = format!("{}/topology.lisp", dir);
    if u::file_exists(&yaml_file) {
        TopologySpec::new(&yaml_file)
    } else if u::file_exists(&lisp_file) {
        let data = u::slurp(&lisp_file);
        lisp::load(data);
        TopologySpec::new(&yaml_file)
    } else {
        panic!("No topology spec found");
    }
}

pub fn compile_root(dir: &str) -> HashMap<String, TopologySpec> {
    let f = format!("{}/topology.yml", dir);
    if u::file_exists(&f) {
        let spec = TopologySpec::new(&f);
        let given_root_dirs = match &spec.nodes.dirs {
            Some(dirs) => dirs,
            None => &u::list_dirs(dir),
        };
        let mut h: HashMap<String, TopologySpec> = HashMap::new();
        if given_root_dirs.is_empty() {
            let tspec = compile(&u::pwd());
            h.insert(tspec.name.clone(), tspec);
        } else {
            for d in given_root_dirs {
                tracing::debug!("Given root: {}", &d);
                let dir = u::absolutize(dir, &d);
                let t = compile(&dir);
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
                let tspec = compile(&d);
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

pub fn pprint(dir: &str, component: &str, format: &str) {
    let versions = lookup_versions(dir);
    match component {
        "versions" => printer::print_versions(versions, format),
        _ => (),
    }
}
