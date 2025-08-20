pub mod build;
pub mod layer;
pub mod runtime;

use super::template;
use crate::{
    Entity,
    spec::{
        ConfigSpec,
        FunctionSpec,
        TestSpec,
    },
};
pub use build::Build;
use kit as u;
use kit::*;
pub use runtime::Runtime;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
    pub skip: bool,
    pub name: String,
    pub actual_name: String,
    pub namespace: String,
    pub dir: String,
    pub description: Option<String>,
    pub fqn: String,
    pub arn: String,
    pub layer_name: Option<String>,
    pub version: String,
    pub runtime: Runtime,
    pub build: Build,
    pub test: HashMap<String, TestSpec>,
    pub targets: Vec<Target>,
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.yml";
    let function_file_json = "function.json";
    let topology_file = "topology.yml";
    (u::file_exists(function_file) || u::file_exists(function_file_json)) && u::file_exists(topology_file)
}


fn find_fqn(given_fqn: &str, namespace: &str, name: &str, format: &str) -> String {
    if !given_fqn.is_empty() {
        format!("{}_{{{{sandbox}}}}", given_fqn)
    } else if !name.is_empty() && namespace.is_empty() {
        format!("{}_{{{{sandbox}}}}", name)
    } else if is_singular_function_dir() {
        format!("{}_{{{{sandbox}}}}", namespace)
    } else {
        match format {
            "hyphenated" => format!("{}-{}-{{{{sandbox}}}}", namespace, name),
            _ => {
                if namespace.is_empty() {
                    format!("{}_{{{{sandbox}}}}", name)
                } else {
                    format!("{}_{}_{{{{sandbox}}}}", namespace, name)
                }
            }
        }
    }
}

fn make_fqn(fspec: &FunctionSpec, namespace: &str, format: &str) -> String {
    match &fspec.fqn {
        Some(f) => find_fqn(&f, namespace, &fspec.name, format),
        None => match &fspec.namespace {
            Some(n) => {
                if format == "hyphenated" {
                    format!("{}-{}-{{{{sandbox}}}}", n, &fspec.name)
                } else {
                    format!("{}_{}_{{{{sandbox}}}}", n, &fspec.name)
                }
            }
            None => {
                if format == "hyphenated" {
                    format!("{}-{}-{{{{sandbox}}}}", namespace, &fspec.name)
                } else {
                    if namespace.is_empty() {
                        format!("{}_{{{{sandbox}}}}", &fspec.name)
                    } else {
                        format!("{}_{}_{{{{sandbox}}}}", namespace, &fspec.name)
                    }
                }
            }
        },
    }
}


fn files_modified() -> Vec<String> {
    match std::env::var("CIRCLE_SHA1") {
        Ok(sha) => {
            let dir = u::pwd();
            let s = format!("git diff --name-only {}^1", sha);
            let out = u::sh(&s, &dir);
            u::split_lines(&out)
                .iter()
                .map(|v| u::absolutize(&dir, v))
                .collect()
        }
        Err(_) => {
            let dir = u::pwd();
            let out = u::sh("git ls-files -m", &dir);
            u::split_lines(&out)
                .iter()
                .map(|v| u::absolutize(&dir, v))
                .collect()
        }
    }
}

fn is_dirty(dir: &str) -> bool {
    let modified = files_modified();
    for m in modified {
        if m.starts_with(dir) {
            return true
       }
    }
    return false
}

fn make_test(t: Option<HashMap<String, TestSpec>>) -> HashMap<String, TestSpec>{
    match t {
        Some(spec) => spec,
        None => HashMap::new()
    }
}

impl Function {
    pub fn new(dir: &str, topo_infra_dir: &str, namespace: &str, format: &str) -> Function {

        let config = ConfigSpec::new(None);

        let fspec = FunctionSpec::new(dir);

        let namespace = match fspec.namespace {
            Some(ref n) => n,
            None => &namespace.to_string(),
        };
        let fqn = make_fqn(&fspec, &namespace, format);

        let infra_dir = match fspec.infra_dir {
            Some(ref d) => &d,
            None => topo_infra_dir,
        };

        let runtime = Runtime::new(dir, infra_dir, &namespace, &fspec, &fqn, &config);

        Function {
            skip: !is_dirty(dir),
            name: fspec.name.to_string(),
            actual_name: fspec.name.to_string(),
            arn: template::lambda_arn(&fqn),
            version: s!(""),
            fqn: fqn.clone(),
            description: None,
            dir: dir.to_string(),
            namespace: namespace.to_string(),
            build: Build::new(dir, &runtime, fspec.build, fspec.tasks),
            layer_name: fspec.layer_name,
            test: make_test(fspec.test),
            runtime: runtime,
            targets: vec![],
        }
    }

    pub fn from_spec(
        fspec: &FunctionSpec,
        namespace: &str,
        dir: &str,
        infra_dir: &str,
    ) -> Function {
        let config = ConfigSpec::new(None);
        let namespace = match fspec.namespace {
            Some(ref n) => n,
            None => &namespace.to_string(),
        };
        let fqn = make_fqn(&fspec, &namespace, "");

        let infra_dir = match fspec.infra_dir {
            Some(ref d) => &d,
            None => infra_dir,
        };

        let runtime = Runtime::new(dir, infra_dir, &namespace, &fspec, &fqn, &config);

        Function {
            skip: !is_dirty(dir),
            name: fspec.name.to_string(),
            actual_name: fspec.name.to_string(),
            arn: template::lambda_arn(&fqn),
            version: s!(""),
            fqn: fqn.clone(),
            description: None,
            dir: dir.to_string(),
            namespace: namespace.to_string(),
            build: Build::new(dir, &runtime, fspec.build.clone(), fspec.tasks.clone()),
            layer_name: fspec.layer_name.clone(),
            test: make_test(fspec.test.clone()),
            runtime: runtime,
            targets: vec![],
        }
    }

    pub fn to_map(function: Function) -> HashMap<String, Function> {
        let mut fns: HashMap<String, Function> = HashMap::new();
        fns.insert(function.name.to_string(), function);
        fns
    }
}
