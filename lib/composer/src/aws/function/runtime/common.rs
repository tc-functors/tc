use kit as u;
use kit::*;
use crate::{index, aws::template};
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use crate::Role;
use compiler::{
    Entity,
    spec::{
        function::{
            RuntimeSpec,
            Arch,
            AssetsSpec,
            BuildKind,
            FileSystemKind,
            FunctionSpec,
            Lang,
            LangRuntime,
            Provider,
            MicroVm
        },
        infra::InfraSpec,
    },
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Network {
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSystem {
    pub kind: FileSystemKind,
    pub arn: String,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Runtime {
    pub lang: LangRuntime,
    pub provider: Provider,
    pub handler: String,
    pub package_type: String,
    pub uri: String,
    pub layers: Vec<String>,
    pub tags: HashMap<String, String>,
    pub environment: HashMap<String, String>,
    pub memory_size: Option<i32>,
    pub cpu: Option<i32>,
    pub timeout: Option<i32>,
    pub arch: Arch,
    pub snapstart: bool,
    pub provisioned_concurrency: Option<i32>,
    pub reserved_concurrency: Option<i32>,
    pub enable_fs: bool,
    pub enable_network: bool,
    pub network: Option<Network>,
    pub fs: Option<FileSystem>,
    pub role: Role,
    pub infra_spec: HashMap<String, InfraSpec>,
    pub microvm: Option<MicroVm>,
    pub port: i32

}

pub fn find_git_sha(dir: &str) -> String {
    sh("git rev-parse --short HEAD", dir)
}

fn _is_branch(dir: &str) -> bool {
    let branch = sh("git branch --show-current", dir);
    !branch.is_empty()
}

pub fn infer_lang(dir: &str) -> LangRuntime {
    let idx = index::get();
    let pe = |name: &str| idx.path_exists(dir, name);
    if pe("handler.py") || pe("pyproject.toml") {
        LangRuntime::Python310
    } else if pe("Cargo.toml") {
        LangRuntime::Rust
    } else if pe("handler.js") || pe("package.json") {
        LangRuntime::Node22
    } else if pe("Gemfile") || pe("handler.rb") {
        LangRuntime::Ruby32
    } else if pe("deps.edn") {
        LangRuntime::Java21
    } else {
        LangRuntime::Python310
    }
}

pub fn is_singular_function_dir() -> bool {
    let function_file = &compiler::spec::function::find_fspec_file(&u::pwd());
    let function_file_json = "function.json";
    let topology_file = "topology.yml";
    (u::file_exists(function_file) || u::file_exists(function_file_json))
        && u::file_exists(topology_file)
}

pub fn find_build_kind(fspec: &FunctionSpec) -> BuildKind {
    match &fspec.build {
        Some(b) => b.kind.clone(),
        None => BuildKind::Code,
    }
}

pub fn as_infra_dir(dir: &str, _infra_dir: &str) -> String {
    let basename = u::basedir(dir).to_string();
    let parent = u::split_first(dir, &format!("/{basename}"));
    parent
        .replace("/topologies/", "/infrastructure/tc/")
        .replace("_", "-")
}

pub fn as_infra_spec_file(infra_dir: &str, rspec: &RuntimeSpec, function_name: &str) -> Option<String> {
    let f = format!("{}/vars/{}.json", infra_dir, function_name);
    let actual_f = follow_path(&f);
    if index::get().file_exists(&actual_f) {
        Some(actual_f)
    } else {
        match &rspec.vars_file {
            Some(p) => Some(follow_path(&p)),
            None => None,
        }
    }
}

pub fn as_str(v: Option<String>, default: &str) -> String {
    match v {
        Some(s) => s.to_string(),
        None => String::from(default),
    }
}

pub fn make_env_vars(
    dir: &str,
    namespace: &str,
    build_kind: BuildKind,
    maybe_assets: Option<AssetsSpec>,
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
        Lang::Ruby => match build_kind {
            BuildKind::Inline => {
                hmap.insert(s!("GEM_PATH"), s!("/var/task/gems/3.2.0"));
                hmap.insert(s!("GEM_HOME"), s!("/var/task/gems/3.2.0"));
                hmap.insert(s!("BUNDLE_CACHE_PATH"), s!("/var/task/vendor/cache"));
                hmap.insert(s!("RUBYLIB"), s!("$RUBYLIB:/var/task/lib"));
                hmap.insert(s!("LD_LIBRARY_PATH"),
                            s!("/var/lang/lib:/lib64:/usr/lib64:/var/runtime:/var/runtime/lib:/var/task:/var/task/lib"));
            }

            _ => {
                hmap.insert(s!("GEM_PATH"), s!("/opt/ruby/gems/3.2.0"));
                hmap.insert(s!("GEM_HOME"), s!("/opt/ruby/gems/3.2.0"));
                hmap.insert(s!("BUNDLE_CACHE_PATH"), s!("/opt/ruby/lib"));
                hmap.insert(s!("RUBYLIB"), s!("$RUBYLIB:/opt/lib"));

                if index::get().path_exists(dir, "Gemfile") {
                    hmap.insert(s!("AWS_LAMBDA_EXEC_WRAPPER"), s!("/opt/ruby/wrapper"));
                }
            }
        },
        Lang::Python => {
            hmap.insert(s!("PYTHONPATH"), format!("/opt/python:/var/runtime",));
            hmap.insert(s!("MODEL_PATH"), format!("/model",));

            // legacy
            if let Some(assets) = maybe_assets {
                let base_deps_path = as_str(assets.base_deps_path, "/var/python");
                let deps_path = as_str(assets.deps_path, "/var/python");
                let model_path = as_str(assets.model_path, "/model");

                hmap.insert(
                    s!("PYTHONPATH"),
                    format!(
                        "/opt/python:/var/runtime:{}/python:{}/python:{}",
                        &base_deps_path, &deps_path, &model_path
                    ),
                );
                hmap.insert(
                    s!("PATH"),
                    format!(
                        "/opt/python:/var/runtime:/model/bin:{}/python:{}/python:{}",
                        &base_deps_path, &deps_path, &model_path
                    ),
                );
                hmap.insert(s!("LD_LIBRARY_PATH"),
                            format!("/var/lang/lib:/lib64:/usr/lib64:/var/runtime:/var/runtime/lib:/var/task:/var/task/lib:/opt/lib:{}/lib:/model/lib", &deps_path));

                hmap.insert(s!("MODEL_PATH"), model_path);
                hmap.insert(s!("DEPS_PATH"), deps_path);
                hmap.insert(s!("BASE_DEPS_PATH"), base_deps_path);
            }
        }
        Lang::Node => match build_kind {
            BuildKind::Inline => {
                hmap.insert(s!("NODE_PATH"), s!("/var/task/node_modules"));
            }
            _ => (),
        },
        _ => (),
    }

    match environment {
        Some(e) => {
            hmap.extend(e);
            hmap
        }
        None => hmap,
    }
}

fn find_parent_function_role(dir: &str) -> Option<String> {
    u::find_self_or_parent_file(dir, "roles/function.json")
}

pub fn lookup_role(
    infra_dir: &str,
    r: &RuntimeSpec,
    namespace: &str,
    _fqn: &str,
    function_name: &str,
) -> Role {
    match &r.role {
        Some(given) => Role::provided(&given),
        None => {
            let path = match &r.role_file {
                Some(f) => Some(follow_path(&f)),
                None => {
                    let f = format!("{}/roles/{}.json", infra_dir, function_name);
                    if index::get().file_exists(&f) {
                        Some(f)
                    } else {
                        if let Some(p) = find_parent_function_role(infra_dir) {
                            Some(p)
                        } else {
                            None
                        }
                    }
                }
            };
            if let Some(p) = path {
                match &r.role_name {
                    Some(name) => Role::new_static(Entity::Function, &p, namespace, &name),
                    None => Role::new(Entity::Function, &p, namespace, function_name),
                }
            } else {
                match std::env::var("TC_LEGACY_ROLES") {
                    Ok(_) => Role::provided_by_entity(Entity::Function),
                    Err(_) => Role::default(Entity::Function),
                }
            }
        }
    }
}

pub fn parent_tags_file(dir: &str) -> Option<String> {
    u::find_parent_file(dir, "tags.json")
}

pub fn load_tags(infra_dir: &str) -> HashMap<String, String> {
    let tags_file = format!("{}/tags.json", infra_dir);
    let parent_file = parent_tags_file(infra_dir);
    if index::get().file_exists(&tags_file) {
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
            None => HashMap::new(),
        }
    }
}

pub fn make_tags(namespace: &str, infra_dir: &str) -> HashMap<String, String> {
    let tc_version = option_env!("PROJECT_VERSION")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();
    let version = u::current_semver(namespace);
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(s!("namespace"), s!(namespace));
    h.insert(s!("sandbox"), template::sandbox());
    h.insert(s!("version"), version);
    h.insert(s!("deployer"), s!("tc"));
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("tc_version"), tc_version);
    let given_tags = load_tags(infra_dir);
    h.extend(given_tags);
    h
}

fn lookup_infraspec_default(infra_dir: &str, function_name: &str) -> HashMap<String, InfraSpec> {
    let f = format!("{}/vars/{}.json", infra_dir, function_name);
    let actual_f = follow_path(&f);
    let infra_spec_file = if index::get().file_exists(&actual_f) {
        Some(actual_f)
    } else {
        None
    };
    InfraSpec::new(infra_spec_file.clone())
}

pub fn make_default(
    dir: &str,
    infra_dir: &str,
    namespace: &str,
    fqn: &str,
    fspec: &FunctionSpec,
) -> Runtime {
    let lang = infer_lang(dir);
    let role = Role::default(Entity::Function);
    let infra_spec = lookup_infraspec_default(infra_dir, &fspec.name);
    let default_infra_spec = infra_spec.get("default").unwrap();

    let InfraSpec {
        memory_size,
        timeout,
        environment,
        ..
    } = default_infra_spec;

    let vars = make_env_vars(
        dir,
        namespace,
        BuildKind::Code,
        fspec.assets.clone(),
        environment.clone(),
        lang.to_lang(),
        fqn,
    );

    Runtime {
        lang: lang,
        provider: Provider::Lambda,
        handler: s!("handler.handler"),
        package_type: s!("zip"),
        uri: format!("{}/lambda.zip", dir),
        layers: vec![],
        environment: vars,
        tags: make_tags(namespace, &infra_dir),
        provisioned_concurrency: default_infra_spec.provisioned_concurrency.clone(),
        reserved_concurrency: default_infra_spec.reserved_concurrency.clone(),
        role: role,
        memory_size: *memory_size,
        cpu: None,
        timeout: *timeout,
        snapstart: false,
        enable_fs: false,
        enable_network: false,
        network: None,
        arch: Arch::X8664,
        fs: None,
        infra_spec: infra_spec,
        microvm: None,
        port: 8080
    }
}
