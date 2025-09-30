use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};

use std::{
    str::FromStr,
};

use crate::Entity;
use crate::spec::template;
use super::build::BuildKind;
use super::AssetsSpec;
use super::InfraSpec;
use super::FunctionSpec;
use crate::spec::role::RoleSpec;
use std::collections::HashMap;
use super::layer;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Lang {
    Python,
    Ruby,
    Go,
    Rust,
    Node,
    Clojure,
}

impl FromStr for Lang {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "python3.10" | "python3.11" | "python3.9" | "python3.12 | python" => Ok(Lang::Python),
            "ruby3.2" | "ruby" | "ruby32 | ruby" => Ok(Lang::Ruby),
            "node22" | "node20" | "node18 | node" | "Node" => Ok(Lang::Node),
            "rust" => Ok(Lang::Rust),
            _ => Ok(Lang::Python),
        }
    }
}

impl Lang {
    pub fn to_str(&self) -> String {
        match self {
            Lang::Python => s!("python"),
            Lang::Ruby => s!("ruby"),
            Lang::Node => s!("node"),
            Lang::Rust => s!("rust"),
            _ => s!("python"),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum LangRuntime {
    #[serde(alias = "python3.9")]
    Python39,
    #[serde(alias = "python3.10")]
    Python310,
    #[serde(alias = "python3.11")]
    Python311,
    #[serde(alias = "python3.12")]
    Python312,
    #[serde(alias = "python3.13")]
    Python313,
    #[serde(alias = "ruby3.2")]
    Ruby32,
    #[serde(alias = "java21")]
    Java21,
    #[serde(alias = "rust")]
    Rust,
    #[serde(alias = "node22")]
    Node22,
    #[serde(alias = "node20")]
    Node20,
}

impl FromStr for LangRuntime {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "python3.13" => Ok(LangRuntime::Python313),
            "python3.12" => Ok(LangRuntime::Python312),
            "python3.11" => Ok(LangRuntime::Python311),
            "python3.10" => Ok(LangRuntime::Python310),
            "python3.9" => Ok(LangRuntime::Python39),
            "ruby3.2" | "ruby" | "ruby32" => Ok(LangRuntime::Ruby32),
            "clojure" | "java21" => Ok(LangRuntime::Java21),
            "rust" => Ok(LangRuntime::Rust),
            "node22" | "Node" => Ok(LangRuntime::Node22),
            "node20" => Ok(LangRuntime::Node20),
            _ => Ok(LangRuntime::Python311),
        }
    }
}

impl LangRuntime {
    pub fn to_str(&self) -> String {
        match self {
            LangRuntime::Python313 => String::from("python3.13"),
            LangRuntime::Python312 => String::from("python3.12"),
            LangRuntime::Python311 => String::from("python3.11"),
            LangRuntime::Python310 => String::from("python3.10"),
            LangRuntime::Python39 => String::from("python3.9"),
            LangRuntime::Ruby32 => String::from("ruby3.2"),
            LangRuntime::Java21 => String::from("java21"),
            LangRuntime::Node22 => String::from("node22"),
            LangRuntime::Node20 => String::from("node20"),
            LangRuntime::Rust => String::from("rust"),
        }
    }

    pub fn to_lang(&self) -> Lang {
        match self {
            LangRuntime::Python313 => Lang::Python,
            LangRuntime::Python312 => Lang::Python,
            LangRuntime::Python311 => Lang::Python,
            LangRuntime::Python310 => Lang::Python,
            LangRuntime::Python39 => Lang::Python,
            LangRuntime::Ruby32 => Lang::Ruby,
            LangRuntime::Java21 => Lang::Clojure,
            LangRuntime::Rust => Lang::Rust,
            LangRuntime::Node20 => Lang::Node,
            LangRuntime::Node22 => Lang::Node,
        }
    }
}


fn default_lang() -> LangRuntime {
    LangRuntime::Python310
}

fn default_handler() -> String {
    s!("handler.handler")
}

fn default_layers() -> Vec<String> {
    vec![]
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Provider {
    Lambda,
    Fargate,
}

impl FromStr for Provider {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lambda" | "Lambda" => Ok(Provider::Lambda),
            "farget" | "Fargate" => Ok(Provider::Fargate),
            _ => Ok(Provider::Lambda),
        }
    }
}

impl Provider {
    pub fn to_str(&self) -> String {
        match self {
            Provider::Lambda => s!("lambda"),
            Provider::Fargate => s!("fargate"),
        }
    }
}

fn default_provider() -> Option<Provider> {
    Some(Provider::Lambda)
}

fn find_git_sha(dir: &str) -> String {
    sh("git rev-parse --short HEAD", dir)
}

fn find_image_tag(dir: &str, namespace: &str) -> String {
    match std::env::var("TC_VERSION_IMAGES") {
        Ok(_) => u::current_semver(namespace),
        Err(_) => find_git_sha(dir),
    }
}

fn as_uri(
    dir: &str,
    namespace: &str,
    name: &str,
    package_type: &str,
    uri: Option<String>,
) -> String {
    match package_type {
        "Image" | "image" | "oci" => match uri {
            Some(u) => u,
            None => {
                let tag = find_image_tag(dir, namespace);
                format!("{{{{repo}}}}:{}_{}_{}", namespace, name, &tag)
            }
        },
        _ => format!("{}/lambda.zip", dir),
    }
}

fn consolidate_layers(
    extensions: Vec<String>,
    given_layers: Vec<String>,
    implicit_layer: Option<String>,
) -> Vec<String> {
    let mut layers: Vec<String> = vec![];
    let mut e: Vec<String> = extensions;
    let mut g: Vec<String> = given_layers;
    layers.append(&mut e);
    layers.append(&mut g);

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

fn is_singular_function_dir() -> bool {
    let function_file = "function.yml";
    let function_file_json = "function.json";
    let topology_file = "topology.yml";
    (u::file_exists(function_file) || u::file_exists(function_file_json))
        && u::file_exists(topology_file)
}

fn find_implicit_layer_name(dir: &str, namespace: &str, fspec: &FunctionSpec) -> Option<String> {
    let given_fqn = &fspec.fqn;
    let given_layer_name = &fspec.layer_name;

    let build_kind = find_build_kind(fspec);
    match given_layer_name {
        Some(name) => Some(name.to_string()),
        None => match build_kind {
            BuildKind::Code => {
                let lang = infer_lang(dir);
                if lang == LangRuntime::Ruby32 && layer::layerable(dir) {
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
            _ => None,
        },
    }
}

fn find_build_kind(fspec: &FunctionSpec) -> BuildKind {
    match &fspec.build {
        Some(b) => b.kind.clone(),
        None => BuildKind::Code,
    }
}

fn as_str(v: Option<String>, default: &str) -> String {
    match v {
        Some(s) => s.to_string(),
        None => String::from(default),
    }
}

fn follow_path(path: &str) -> String {
    if path.starts_with("..") {
        u::absolutize(&u::pwd(), path)
    } else {
        s!(path)
    }
}

fn as_infra_spec_file(infra_dir: &str, rspec: &RuntimeSpec, function_name: &str) -> Option<String> {
    let f = format!("{}/vars/{}.json", infra_dir, function_name);
    let actual_f = follow_path(&f);
    if u::file_exists(&actual_f) {
        Some(actual_f)
    } else {
        match &rspec.vars_file {
            Some(p) => Some(follow_path(&p)),
            None => None,
        }
    }
}

fn lookup_infraspec(
    infra_dir: &str,
    name: &str,
    rspec: &RuntimeSpec,
) -> HashMap<String, InfraSpec> {
    let infra_spec_file = as_infra_spec_file(&infra_dir, rspec, name);
    InfraSpec::new(infra_spec_file.clone())
}

pub fn lookup_role(
    namespace: &str,
    infra_dir: &str,
    r: &RuntimeSpec,
    function_name: &str,
) -> RoleSpec {
    match &r.role {
        Some(given) => RoleSpec::provided(&given),
        None => {
            let path = match &r.role_file {
                Some(f) => Some(follow_path(&f)),
                None => {
                    let f = format!("{}/roles/{}.json", infra_dir, function_name);
                    if u::file_exists(&f) {
                        Some(f)
                    } else {
                        u::any_path(
                            vec![format!("{}/roles/function.json", infra_dir)]
                        )
                    }
                }
            };

            if let Some(p) = path {
                match &r.role_name {
                    Some(name) => RoleSpec::new_static(Entity::Function, &p, &name),
                    None => RoleSpec::new(Entity::Function, &p, namespace, function_name),
                }
            } else {
                match std::env::var("TC_LEGACY_ROLES") {
                    Ok(_) => RoleSpec::provided_by_entity(Entity::Function),
                    Err(_) => RoleSpec::default(Entity::Function),
                }
            }
        }
    }
}

fn make_env_vars(
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

                if u::path_exists(dir, "Gemfile") {
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

fn as_infra_dir(dir: &str, _infra_dir: &str) -> String {
    let basename = u::basedir(dir).to_string();
    let parent = u::split_first(dir, &format!("/{basename}"));
    parent
        .replace("/topologies/", "/infrastructure/tc/")
        .replace("_", "-")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeSpec {
    #[serde(default = "default_lang")]
    pub lang: LangRuntime,

    #[serde(default = "default_provider")]
    pub provider: Option<Provider>,

    #[serde(default = "default_handler")]
    pub handler: String,
    pub package_type: Option<String>,
    pub uri: Option<String>,
    pub environment: Option<HashMap<String, String>>,
    pub memory_size: Option<i32>,
    pub cpu: Option<i32>,
    pub timeout: Option<i32>,

    pub vars_file: Option<String>,
    pub role_file: Option<String>,
    pub role_name: Option<String>,
    pub role_kind: Option<String>,
    pub role: Option<String>,
    pub role_spec: Option<RoleSpec>,
    pub provisioned_concurrency: Option<i32>,
    pub reserved_concurrency: Option<i32>,
    pub mount_fs: Option<bool>,
    pub snapstart: Option<bool>,
    pub infra_spec: Option<HashMap<String, InfraSpec>>,

    #[serde(default = "default_layers")]
    pub layers: Vec<String>,
    #[serde(default = "default_layers")]
    pub extensions: Vec<String>,
}

impl RuntimeSpec {

    pub fn augment(&self,
                   namespace: &str,
                   fqn: &str,
                   fspec: &FunctionSpec,
                   dir: &str,
                   t_infra_dir: &str,
                   infra_dir: Option<String>
    ) -> RuntimeSpec {

        let infra_dir = match infra_dir {
            Some(p) => p.to_string(),
            None => as_infra_dir(dir, t_infra_dir),
        };

        let layer_name = find_implicit_layer_name(dir, namespace, fspec);

        let layers = consolidate_layers(self.extensions.clone(), self.layers.clone(), layer_name);
        let build_kind = find_build_kind(&fspec);
        let package_type = match &self.package_type {
            Some(x) => x.to_string(),
            None => match build_kind {
                BuildKind::Image => s!("image"),
                _ => s!("zip"),
            },
        };
        let uri = as_uri(dir, namespace, &fspec.name, &package_type, self.uri.clone());

        let infra_spec = lookup_infraspec(&infra_dir, &fspec.name, self);
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
            build_kind,
            fspec.assets.clone(),
            environment.clone(),
            self.lang.to_lang(),
            fqn,
        );

        let role_spec = lookup_role(namespace, &infra_dir, &self, &fspec.name);

        let mut rs = self.clone();

        rs.uri = Some(uri);
        rs.layers = layers;
        rs.environment = Some(vars);

        rs.provisioned_concurrency = default_infra_spec.provisioned_concurrency.clone();
        rs.reserved_concurrency = default_infra_spec.reserved_concurrency.clone();
        rs.memory_size = *memory_size;
        rs.timeout = *timeout;
        rs.role_spec = Some(role_spec);
        rs
    }
}
