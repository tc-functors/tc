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
    pub infra_spec_file: Option<String>,
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
    if u::path_exists(dir, "handler.py") ||
        u::path_exists(dir, "pyproject.toml") {
        LangRuntime::Python310

    } else if u::path_exists(dir, "Cargo.toml") {
        LangRuntime::Rust

    } else if u::path_exists(dir, "handler.js") ||
        u::path_exists(dir, "package.json") {
        LangRuntime::Node22

    } else if u::path_exists(dir, "Gemfile") ||
        u::path_exists(dir, "handler.rb") {
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

fn as_infra_dir(dir: &str, _infra_dir: &str) -> String {
    let basename = u::basedir(dir).to_string();
    let parent = u::split_first(dir, &format!("/{basename}"));
    parent
        .replace("/services/", "/infrastructure/tc/")
        .replace("_", "-")
}


fn as_infra_spec_file(infra_dir: &str, rspec: &Option<RuntimeSpec>, function_name: &str) -> Option<String> {
    let f = format!("{}/vars/{}.json", infra_dir, function_name);
    let actual_f =  follow_path(&f);
    if u::file_exists(&actual_f) {
        Some(actual_f)
    } else {
        match rspec {
            Some(r) => match &r.vars_file {
                Some(p) => Some(follow_path(&p)),
                None => None
            },
            None => None
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


fn make_env_vars(
    dir: &str,
    namespace: &str,
    assets: HashMap<String, Value>,
    environment: Option<HashMap<String, String>>,
    lang: Lang,
    fqn: &str,
) -> HashMap<String, String> {

    let mut hmap: HashMap<String, String> = HashMap::new();

    let mn = u::pascal_case(&format!("{} {}", namespace, fqn));

    hmap.insert(String::from("LAMBDA_STAGE"), template::profile());
    hmap.insert(String::from("Environment"), template::profile());
    hmap.insert(String::from("AWS_ACCOUNT_ID"), template::account());
    hmap.insert(String::from("SANDBOX"), template::sandbox());
    hmap.insert(String::from("NAMESPACE"), s!(namespace));
    hmap.insert(String::from("LOG_LEVEL"), s!("INFO"));
    hmap.insert(String::from("POWERTOOLS_METRICS_NAMESPACE"), mn);

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

            hmap.insert(s!("MODEL_PATH"), model_path);
            hmap.insert(s!("DEPS_PATH"), deps_path);
            hmap.insert(s!("BASE_DEPS_PATH"), base_deps_path);

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

fn parent_tags_file(dir: &str) -> Option<String> {
    let paths = vec![
        u::absolutize(dir, "../tags.json"),
        u::absolutize(dir, "../../tags.json"),
        u::absolutize(dir, "../../../tags.json"),
        u::absolutize(dir, "../../../../tags.json"),
        s!("../tags.json"),
        s!("../../tags.json"),
        s!("../../../tags.json"),
        s!("../../../../tags.json"),
    ];
    u::any_path(paths)
}

fn load_tags(infra_dir: &str) -> HashMap<String, String> {
    let tags_file = format!("{}/tags.json", infra_dir);
    let parent_file = parent_tags_file(infra_dir);
    if u::file_exists(&tags_file) {
        let data: String = u::slurp(&tags_file);
        let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
        tags
    } else {
        match parent_file {
            Some(f) => {
                let data: String = u::slurp(&f);
                let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
                tags
            }
            None => {
                HashMap::new()
            }
        }
    }
}

fn make_tags(namespace: &str, infra_dir: &str) -> HashMap<String, String> {
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
    let given_tags = load_tags(infra_dir);
    h.extend(given_tags);
    h
}

fn needs_fs(assets: HashMap<String, Value>, mount_fs: Option<bool>) -> bool {
    let ax = assets.get("DEPS_PATH");
    match ax {
        Some(_) => true,
        None => match mount_fs {
            Some(f) => f,
            None => {
                match assets.get("MODEL_PATH") {
                    Some(_) => true,
                    None => false
                }
            }
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

    pub fn new(dir: &str, t_infra_dir: &str, namespace: &str, fspec: &FunctionSpec, fqn: &str) -> Runtime {
        let rspec = fspec.runtime.clone();

        let infra_dir = match &fspec.infra_dir {
            Some(p) => p.to_string(),
            None => as_infra_dir(dir, t_infra_dir)
        };
        let infra_spec_file = as_infra_spec_file(&infra_dir, &rspec, &fspec.name);

        let infra_spec = RuntimeInfraSpec::new(infra_spec_file.clone());
        //FIXME: handle unwrap
        let default_infra_spec = infra_spec.get("default").unwrap();
        let RuntimeInfraSpec { memory_size, timeout, ref environment, .. } = default_infra_spec;

        let role = lookup_role(&infra_dir, &rspec, namespace, &fspec.name);

        match rspec {
            Some(mut r) => {
                let layer_name = find_layer_name(dir, namespace, fspec);
                let layers = consolidate_layers(&mut r.extensions, &mut r.layers, layer_name);
                let package_type = &r.package_type;
                let vars = make_env_vars(
                    dir,
                    namespace,
                    fspec.assets.clone(),
                    environment.clone(),
                    r.lang.to_lang(),
                    fqn
                );

                let enable_fs = needs_fs(fspec.assets.clone(), r.mount_fs);
                Runtime {
                    lang: r.lang,
                    handler: r.handler,
                    package_type: package_type.to_string(),
                    uri: as_uri(dir, package_type, r.uri),
                    layers: layers,
                    tags: make_tags(namespace, &infra_dir),
                    environment: vars,
                    provisioned_concurrency: None,
                    memory_size: *memory_size,
                    timeout: *timeout,
                    snapstart: u::opt_as_bool(r.snapstart),
                    role: role,
                    enable_fs: enable_fs,
                    network: make_network(&default_infra_spec, enable_fs),
                    fs: make_fs(&default_infra_spec, enable_fs),
                    infra_spec_file: infra_spec_file,
                    infra_spec: infra_spec
                }
            },
            None => {
                let lang = infer_lang(dir);
                let vars = make_env_vars(
                    dir,
                    namespace,
                    fspec.assets.clone(),
                    environment.clone(),
                    lang.to_lang(),
                    fqn
                );

                Runtime {
                    lang: lang,
                    handler: s!("handler.handler"),
                    package_type: s!("zip"),
                    uri: as_uri(dir, "zip", None),
                    layers: vec![],
                    environment: vars,
                    tags: make_tags(namespace, &infra_dir),
                    provisioned_concurrency: None,
                    role: role,
                    memory_size: *memory_size,
                    timeout: *timeout,
                    snapstart: false,
                    enable_fs: false,
                    network: None,
                    fs: None,
                    infra_spec_file: infra_spec_file,
                    infra_spec: infra_spec
                }
            }
        }
    }
}
