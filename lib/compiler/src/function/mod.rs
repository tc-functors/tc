pub mod build;
pub mod runtime;
pub mod layer;
pub mod role;

use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use super::spec::FunctionSpec;
use crate::template;
pub use build::Build;
pub use runtime::Runtime;
pub use role::Role;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Test {
    pub name: String,
    pub commands: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
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
    pub test: Test,
    pub assets: HashMap<String, Value>,
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    u::file_exists(function_file) && u::file_exists(topology_file)
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
            _ => format!("{}_{}_{{{{sandbox}}}}", namespace, name),
        }
    }
}

fn make_test() -> Test {
    Test {
        name: u::empty(),
        commands: vec![]
    }
}

fn make_fqn(fspec: &FunctionSpec, namespace: &str, format: &str) -> String {
    match &fspec.fqn {
        Some(f) => {
            find_fqn(&f, namespace, &fspec.name, format)
        },
        None => match &fspec.namespace {
            Some(n) => format!("{}_{}_{{{{sandbox}}}}", n, &fspec.name),
            None => format!("{}_{{{{sandbox}}}}", &fspec.name)
        }
    }
}

impl Function {

    pub fn new(dir: &str, infra_dir: &str, namespace: &str, format: &str) -> Function {
        let fspec = FunctionSpec::new(dir);

        let namespace = match fspec.namespace {
            Some(ref n) => n,
            None => &namespace.to_string()
        };
        let runtime = Runtime::new(dir, infra_dir,  &namespace, &fspec);
        let fqn = make_fqn(&fspec, &namespace, format);

        Function {
            name: fspec.name.to_string(),
            actual_name: fspec.name.to_string(),
            arn: template::lambda_arn(&fqn),
            version: s!(""),
            fqn: fqn.clone(),
            description: None,
            dir: fspec.dir.clone(),
            namespace: namespace.to_string(),
            build: Build::new(&runtime, fspec.build),
            runtime: runtime,
            layer_name: fspec.layer_name,
            assets: fspec.assets,
            test: make_test()
        }
    }

    pub fn to_map(function: Function) -> HashMap<String, Function> {
        let mut fns: HashMap<String, Function> = HashMap::new();
        fns.insert(function.dir.to_string(), function);
        fns
    }
}
