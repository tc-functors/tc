use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;
use crate::spec::{LangRuntime, FunctionSpec, RuntimeInfraSpec, RuntimeSpec, Lang};
use crate::{version, template, role};
use super::{layer};
use crate::role::{Role, RoleKind};

use kit as u;
use kit::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Network {
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSystem {
    pub arn: String,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Runtime {
    pub lang: LangRuntime,
    pub handler: String,
    pub package_type: String,
    pub uri: String,
    pub layers: Vec<String>,
    pub tags: HashMap<String, String>,
    pub environment: HashMap<String, String>,
    pub memory_size: Option<i32>,
    pub timeout: Option<i32>,
    pub snapstart: bool,
    pub provisioned_concurrency: Option<i32>,
    pub enable_fs: bool,
    pub network: Option<Network>,
    pub fs: Option<FileSystem>,
    pub role: Role,
    pub infra_spec: HashMap<String, RuntimeInfraSpec>
}

fn as_uri(dir: &str, package_type: &str, uri: Option<String>) -> String {
    match package_type {
        "image" | "oci" => match uri {
            Some(u) => u,
            None => format!("TEMPLATED")
        },
        _ => format!("{}/lambda.zip", dir)
    }
}

fn consolidate_layers(
    extensions: &mut Vec<String>,
    given_layers: &mut Vec<String>,
    implicit_layer: Option<String>,
) -> Vec<String> {

    let mut layers: Vec<String> = vec![];
    layers.append(given_layers);
    layers.append(extensions);
    match implicit_layer {
        Some(m) => layers.push(m),
        None => (),
    }
    u::uniq(layers)
}

pub fn infer_lang(dir: &str) -> LangRuntime {
    if u::path_exists(dir, "handler.py") || u::path_exists(dir, "pyproject.toml") {
        LangRuntime::Python310
    } else if u::path_exists(dir, "Cargo.toml") {
        LangRuntime::Rust
    } else if u::path_exists(dir, "Gemfile") || u::path_exists(dir, "handler.rb") {
        LangRuntime::Ruby32
    } else if u::path_exists(dir, "deps.edn") {
        LangRuntime::Java21
    } else {
        LangRuntime::Python310
    }
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    u::file_exists(function_file) && u::file_exists(topology_file)
}

fn find_layer_name(dir: &str, namespace: &str, fspec: &FunctionSpec) -> Option<String> {
    let given_fqn = &fspec.fqn;
    let given_layer_name = &fspec.layer_name;

    match given_layer_name {
        Some(name) => Some(name.to_string()),
        None => {
            let lang = infer_lang(dir);
            if lang == LangRuntime::Ruby32  && layer::layerable(dir) {
                match given_fqn {
                    Some(f) => Some(u::kebab_case(&f)),
                    None => {
                        if is_singular_function_dir() {
                            Some(s!(namespace))
                        } else {
                            Some(format!("{}-{}", namespace, &fspec.name))
                        }
                    }
                }
            } else {
                None
            }
        }
    }
}

fn follow_path(path: &str) -> String {
    if path.starts_with("..") {
        u::absolutize(&u::pwd(), path)
    } else {
        s!(path)
    }
}

fn lookup_infra_spec(infra_dir: &str, rspec: &Option<RuntimeSpec>, function_name: &str, ) -> HashMap<String, RuntimeInfraSpec> {

    let f = format!("{}/vars/{}.json", infra_dir, function_name);
    let actual_f =  follow_path(&f);
    if u::file_exists(&actual_f) {
        RuntimeInfraSpec::new(Some(actual_f))
    } else {
        match rspec {
            Some(r) => {
                match &r.vars_file {
                    Some(f) => RuntimeInfraSpec::new(Some(follow_path(&f))),
                    None => RuntimeInfraSpec::new(None)
                }
            },
            None => RuntimeInfraSpec::new(None)
        }
    }
}

fn lookup_role(infra_dir: &str, rspec: &Option<RuntimeSpec>, namespace: &str, function_name: &str) -> Role {
    match rspec {
        Some(r) => {
            let path = match &r.role_file {
                Some(f) => Some(follow_path(&f)),
                None => {
                    let f = format!("{}/roles/{}.json", infra_dir, function_name);
                    if u::file_exists(&f) {
                        Some(f)
                    } else {
                        None
                    }
                }
            };

            if let Some(p) = path {
                let abbr = if function_name.chars().count() > 15 {
                    u::abbreviate(function_name, "-")
                } else {
                    function_name.to_string()
                };
                let policy_name = format!("tc-{}-{{{{sandbox}}}}-{}-policy", namespace, abbr);
                let role_name = format!("tc-{}-{{{{sandbox}}}}-{}-role", namespace, abbr);

                Role::new(RoleKind::Function, &p, &role_name, &policy_name)
            } else {
                role::default(RoleKind::Function)
            }

        },
        None => role::default(RoleKind::Function)
    }
}

fn value_to_str(v: Option<&Value>, default: &str) -> String {
    match v {
        Some(s) => s.as_str().unwrap().to_string(),
        None => String::from(default)
    }
}

fn make_env_vars(dir: &str,
                 namespace: &str,
                 assets: HashMap<String, Value>,
                 environment: Option<HashMap<String, String>>,
                 lang: Lang,

) -> HashMap<String, String> {

    let mut hmap: HashMap<String, String> = HashMap::new();

    hmap.insert(String::from("LAMBDA_STAGE"), template::profile());
    hmap.insert(String::from("Environment"), template::profile());
    hmap.insert(String::from("AWS_ACCOUNT_ID"), template::account());
    hmap.insert(String::from("SANDBOX"), template::sandbox());
    hmap.insert(String::from("NAMESPACE"), s!(namespace));
    hmap.insert(String::from("LOG_LEVEL"), s!("INFO"));

    match lang {
        Lang::Ruby => {
            hmap.insert(s!("GEM_PATH"), s!("/opt/ruby/gems/3.2.0"));
            hmap.insert(s!("GEM_HOME"), s!("/opt/ruby/gems/3.2.0"));
            hmap.insert(s!("BUNDLE_CACHE_PATH"), s!("/opt/ruby/lib"));
            hmap.insert(s!("RUBYLIB"), s!("$RUBYLIB:/opt/lib"));

            match std::env::var("NO_RUBY_WRAPPER") {
                Ok(_) => (),
                Err(_) => {
                    if u::path_exists(dir, "Gemfile") {
                        hmap.insert(s!("AWS_LAMBDA_EXEC_WRAPPER"), s!("/opt/ruby/wrapper"));
                    }
                }
            }
        },
        Lang::Python => {
            // legacy
            let base_deps_path = value_to_str(assets.get("BASE_DEPS_PATH"), "/var/python");
            let deps_path = value_to_str(assets.get("DEPS_PATH"), "/var/python");
            let model_path = value_to_str(assets.get("MODEL_PATH"), "/var/python");

            hmap.insert(s!("PYTHONPATH"),
                        format!(
                            "/opt/python:/var/runtime:{}/python:{}/python:{}",
                            &base_deps_path, &deps_path, &model_path
                        ),
            );
            hmap.insert(s!("LD_LIBRARY_PATH"),
                        format!("/var/lang/lib:/lib64:/usr/lib64:/var/runtime:/var/runtime/lib:/var/task:/var/task/lib:/opt/lib:{}/lib", &deps_path));

        },
        _ => ()
    }

    match environment {
        Some(e) => {
            hmap.extend(e);
            hmap
        }
        None => hmap
    }
}

fn make_tags(namespace: &str) -> HashMap<String, String> {
    let tc_version = option_env!("PROJECT_VERSION")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();
    let version = version::current_semver(namespace);
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(s!("namespace"), s!(namespace));
    h.insert(s!("sandbox"), template::sandbox());
    h.insert(s!("version"), version);
    h.insert(s!("git_branch"), version::branch_name());
    h.insert(s!("deployer"), s!("tc"));
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("tc_version"), tc_version);
    h
}

fn needs_fs(assets: HashMap<String, Value>, mount_fs: Option<bool>) -> bool {
    let assets = assets.get("MODEL_PATH");
    match assets {
        Some(_) => true,
        None => match mount_fs {
            Some(f) => f,
            None => false
        }
    }
}

fn make_network(infra_spec: &RuntimeInfraSpec, enable_fs: bool) -> Option<Network> {
    if enable_fs {
        match &infra_spec.network {
            Some(net) => Some(Network {
                subnets: net.subnets.clone(),
                security_groups: net.security_groups.clone()
            }),
            None => None
        }
    } else {
        None
    }
}

fn make_fs(infra_spec: &RuntimeInfraSpec, enable_fs: bool) -> Option<FileSystem> {
    if enable_fs {
        match &infra_spec.filesystem {
            Some(fs) => Some(FileSystem {
                arn: fs.arn.clone(),
                mount_point: fs.mount_point.clone()
            }),
            None => None
        }
    } else {
        None
    }
}

impl Runtime {

    pub fn new(dir: &str, infra_dir: &str, namespace: &str, fspec: &FunctionSpec) -> Runtime {
        let rspec = fspec.runtime.clone();

        let infra_spec = lookup_infra_spec(infra_dir, &rspec, &fspec.name);
        //FIXME: handle unwrap
        let default_infra_spec = infra_spec.get("default").unwrap();
        let RuntimeInfraSpec { memory_size, timeout, ref environment, .. } = default_infra_spec;

        let role = lookup_role(infra_dir, &rspec, namespace, &fspec.name);

        match rspec {
            Some(mut r) => {
                let layer_name = find_layer_name(dir, namespace, fspec);
                let layers = consolidate_layers(&mut r.extensions, &mut r.layers, layer_name);
                let package_type = &r.package_type;
                let vars = make_env_vars(dir, namespace, fspec.assets.clone(), environment.clone(), r.lang.to_lang());

                let enable_fs = needs_fs(fspec.assets.clone(), r.mount_fs);
                Runtime {
                    lang: r.lang,
                    handler: r.handler,
                    package_type: package_type.to_string(),
                    uri: as_uri(dir, package_type, r.uri),
                    layers: layers,
                    tags: make_tags(namespace),
                    environment: vars,
                    provisioned_concurrency: None,
                    memory_size: *memory_size,
                    timeout: *timeout,
                    snapstart: false,
                    role: role,
                    enable_fs: enable_fs,
                    network: make_network(&default_infra_spec, enable_fs),
                    fs: make_fs(&default_infra_spec, enable_fs),
                    infra_spec: infra_spec
                }
            },
            None => {
                let lang = infer_lang(dir);
                let vars = make_env_vars(dir, namespace, fspec.assets.clone(), environment.clone(), lang.to_lang());

                Runtime {
                    lang: lang,
                    handler: s!("handler.handler"),
                    package_type: s!("zip"),
                    uri: as_uri(dir, "zip", None),
                    layers: vec![],
                    environment: vars,
                    tags: make_tags(namespace),
                    provisioned_concurrency: None,
                    role: role,
                    memory_size: *memory_size,
                    timeout: *timeout,
                    snapstart: false,
                    enable_fs: false,
                    network: None,
                    fs: None,
                    infra_spec: infra_spec
                }
            }
        }
    }
}
