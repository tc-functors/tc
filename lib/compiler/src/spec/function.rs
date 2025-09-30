use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashMap,
};
pub mod runtime;
pub mod build;
pub mod layer;
pub mod infra;

use crate::spec::TestSpec;
pub use runtime::{RuntimeSpec, Lang, LangRuntime};
pub use build::BuildSpec;
pub use infra::InfraSpec;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AssetsSpec {
    #[serde(alias = "DEPS_PATH", alias = "deps_path")]
    pub deps_path: Option<String>,
    #[serde(alias = "BASE_DEPS_PATH", alias = "base_deps_path")]
    pub base_deps_path: Option<String>,
    #[serde(alias = "MODEL_PATH", alias = "model_path")]
    pub model_path: Option<String>,
    #[serde(alias = "ARTIFACTS_SOURCE", alias = "artifacts_source")]
    pub artifacts_source: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionSpec {
    pub name: String,
    pub uri: Option<String>,
    pub root: Option<bool>,
    pub dir: Option<String>,
    pub description: Option<String>,
    pub namespace: Option<String>,
    pub fqn: Option<String>,
    pub layer_name: Option<String>,
    pub version: Option<String>,
    pub revision: Option<String>,
    pub runtime: Option<RuntimeSpec>,
    pub build: Option<BuildSpec>,
    #[serde(alias = "tests")]
    pub test: Option<HashMap<String, TestSpec>>,
    //deprecated
    pub infra_dir: Option<String>,
    //deprecated
    #[serde(default)]
    pub tasks: HashMap<String, String>,
    //deprecated
    pub assets: Option<AssetsSpec>,
    // flow
    pub function: Option<String>,
}

fn find_revision(dir: &str) -> String {
    let cmd_str = format!("git log -n 1 --format=%h {}", dir);
    u::sh(&cmd_str, dir)
}

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.yml";
    let function_file_json = "function.json";
    let topology_file = "topology.yml";
    (u::file_exists(function_file) || u::file_exists(function_file_json))
        && u::file_exists(topology_file)
}

fn render(s: &str, version: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    let root = &top_level();
    table.insert("version", version);
    table.insert("root", root);
    table.insert("git_root", root);
    table.insert("sandbox", "{{sandbox}}");
    table.insert("repo", "{{repo}}");
    table.insert("account", "{{account}}");
    table.insert("region", "{{region}}");
    u::stencil(s, table)
}

fn load_fspec_file(version: &str, dir: &str) -> Option<FunctionSpec> {
    let f1 = format!("{}/function.json", dir);
    let f2 = format!("{}/function.yml", dir);
    let f3 = format!("{}/function.yaml", dir);
    if u::file_exists(&f1) {
        let data = render(&u::slurp(&f1), &version);
        let fspec: Result<FunctionSpec, _> = serde_json::from_str(&data);
        match fspec {
            Ok(f) => Some(f),
            Err(e) => panic!("{:?}", e),
        }
    } else if u::file_exists(&f2) {
        let data = render(&u::slurp(&f2), &version);
        let fspec: Result<FunctionSpec, _> = serde_yaml::from_str(&data);
        match fspec {
            Ok(f) => Some(f),
            Err(e) => panic!("{:?}", e),
        }
    } else if u::file_exists(&f3) {
        let data = render(&u::slurp(&f3), &version);
        let fspec: Result<FunctionSpec, _> = serde_yaml::from_str(&data);
        match fspec {
            Ok(f) => Some(f),
            Err(e) => panic!("{:?}", e),
        }
    } else {
        None
    }
}


fn find_fqn(given_fqn: &str, namespace: &str, name: &str) -> String {
    if !given_fqn.is_empty() {
        format!("{}_{{{{sandbox}}}}", given_fqn)
    } else if !name.is_empty() && namespace.is_empty() {
        format!("{}_{{{{sandbox}}}}", name)
    } else if is_singular_function_dir() {
        format!("{}_{{{{sandbox}}}}", namespace)
    } else {
        if namespace.is_empty() {
            format!("{}_{{{{sandbox}}}}", name)
        } else {
            format!("{}_{}_{{{{sandbox}}}}", namespace, name)
        }
    }
}

fn make_fqn(fspec: &FunctionSpec, namespace: &str) -> String {
    match &fspec.fqn {
        Some(f) => find_fqn(&f, namespace, &fspec.name),
        None => match &fspec.namespace {
            Some(n) => {
                format!("{}_{}_{{{{sandbox}}}}", n, &fspec.name)
            }
            None => {
                if namespace.is_empty() {
                    format!("{}_{{{{sandbox}}}}", &fspec.name)
                } else {
                    format!("{}_{}_{{{{sandbox}}}}", namespace, &fspec.name)
                }
            }
        }
    }
}


impl FunctionSpec {
    pub fn new(dir: &str) -> FunctionSpec {
        let version = find_revision(dir);
        let maybe_spec = load_fspec_file(&version, dir);

        match maybe_spec {
            Some(f) => f,
            None => FunctionSpec {
                name: u::basedir(dir).to_string(),
                dir: Some(dir.to_string()),
                uri: None,
                root: None,
                description: None,
                namespace: None,
                fqn: None,
                layer_name: None,
                version: None,
                revision: None,
                runtime: None,
                build: None,
                infra_dir: None,
                assets: None,
                test: None,
                tasks: HashMap::new(),
                function: None
            },
        }
    }

    pub fn intern(&self, namespace: &str, dir: &str, infra_dir: &str, name: &str) -> FunctionSpec {

        FunctionSpec {
            name: s!(name),
            dir: Some(s!(dir)),
            root: self.root.clone(),
            uri: self.uri.clone(),
            description: None,
            namespace: Some(s!(namespace)),
            fqn: self.fqn.clone(),
            layer_name: None,
            version: None,
            revision: None,
            runtime: self.runtime.clone(),
            build: self.build.clone(),
            assets: None,
            infra_dir: Some(s!(infra_dir)),
            test: None,
            tasks: HashMap::new(),
            function: None
        }
    }


    pub fn augment(&self, namespace: &str, dir: &str, t_infra_dir: &str, infra_dir: Option<String>) -> FunctionSpec {
        let mut fs = self.clone();
        let fqn = make_fqn(&self, namespace);
        let namespace = match self.namespace {
            Some(ref n) => n,
            None => &namespace.to_string(),
        };
        fs.namespace = Some(namespace.clone());
        match &self.build {
            Some(b) => {
                let package_type = self.runtime.clone().unwrap().package_type;
                let package_type = u::maybe_string(package_type, "zip");
                let build = b.augment(&package_type);
                fs.build = Some(build);
            },
            None => {
                let build = BuildSpec::default(&self.tasks);
                fs.build = Some(build);
            }
        }

        if let Some(r) = &self.runtime {
            let runtime = r.augment(namespace, &fqn, self, dir, t_infra_dir, infra_dir);
            fs.runtime = Some(runtime);
        }
        fs.fqn = Some(fqn);
        fs
    }
}


pub fn infer_lang(dir: &str) -> LangRuntime {
    if u::path_exists(dir, "handler.py") || u::path_exists(dir, "pyproject.toml") {
        LangRuntime::Python310
    } else if u::path_exists(dir, "Cargo.toml") {
        LangRuntime::Rust
    } else if u::path_exists(dir, "handler.js") || u::path_exists(dir, "package.json") {
        LangRuntime::Node22
    } else if u::path_exists(dir, "Gemfile") || u::path_exists(dir, "handler.rb") {
        LangRuntime::Ruby32
    } else if u::path_exists(dir, "deps.edn") {
        LangRuntime::Java21
    } else {
        LangRuntime::Python310
    }
}
