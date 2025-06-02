use doku::Document;
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashMap,
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Document)]
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
            "node22" | "node20" | "node18 | node" => Ok(Lang::Node),
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Document)]
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
            "python3.13"                  => Ok(LangRuntime::Python313),
            "python3.12"                  => Ok(LangRuntime::Python312),
            "python3.11"                  => Ok(LangRuntime::Python311),
            "python3.10"                  => Ok(LangRuntime::Python310),
            "python3.9"                   => Ok(LangRuntime::Python39),
            "ruby3.2" | "ruby" | "ruby32" => Ok(LangRuntime::Ruby32),
            "clojure" | "java21"          => Ok(LangRuntime::Java21),
            "rust"                        => Ok(LangRuntime::Rust),
            "node22"                      => Ok(LangRuntime::Node22),
            "node20"                      => Ok(LangRuntime::Node20),
            _                             => Ok(LangRuntime::Python311),
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
            LangRuntime::Python39  => String::from("python3.9"),
            LangRuntime::Ruby32    => String::from("ruby3.2"),
            LangRuntime::Java21    => String::from("java21"),
            LangRuntime::Node22    => String::from("node22"),
            LangRuntime::Node20    => String::from("node20"),
            LangRuntime::Rust      => String::from("rust"),
        }
    }

    pub fn to_lang(&self) -> Lang {
        match self {
            LangRuntime::Python313 => Lang::Python,
            LangRuntime::Python312 => Lang::Python,
            LangRuntime::Python311 => Lang::Python,
            LangRuntime::Python310 => Lang::Python,
            LangRuntime::Python39  => Lang::Python,
            LangRuntime::Ruby32    => Lang::Ruby,
            LangRuntime::Java21    => Lang::Clojure,
            LangRuntime::Rust      => Lang::Rust,
            LangRuntime::Node20    => Lang::Node,
            LangRuntime::Node22    => Lang::Node,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Document, PartialEq, Eq)]
pub enum BuildKind {
    #[serde(alias = "code")]
    Code,
    #[serde(alias = "inline")]
    Inline,
    #[serde(alias = "layer")]
    Layer,
    #[serde(alias = "slab")]
    Slab,
    #[serde(alias = "library")]
    Library,
    #[serde(alias = "extension")]
    Extension,
    #[serde(alias = "runtime")]
    Runtime,
    #[serde(alias = "image")]
    Image,
}

impl FromStr for BuildKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(BuildKind::Code),
            "inline" => Ok(BuildKind::Inline),
            "layer" => Ok(BuildKind::Layer),
            "library" => Ok(BuildKind::Library),
            "extension" => Ok(BuildKind::Extension),
            "runtime" => Ok(BuildKind::Runtime),
            "slab" => Ok(BuildKind::Slab),
            "image" => Ok(BuildKind::Image),
            _ => Ok(BuildKind::Layer),
        }
    }
}

impl BuildKind {
    pub fn to_str(&self) -> String {
        match self {
            BuildKind::Code => s!("code"),
            BuildKind::Inline => s!("inline"),
            BuildKind::Layer => s!("layer"),
            BuildKind::Library => s!("library"),
            BuildKind::Extension => s!("extension"),
            BuildKind::Runtime => s!("runtime"),
            BuildKind::Image => s!("image"),
            BuildKind::Slab => s!("slab"),
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

fn default_command() -> String {
    s!("zip -9 -r lambda.zip .")
}

fn default_infra_dir() -> String {
    u::empty()
}

fn default_package_type() -> String {
    s!("zip")
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct ImageSpec {
    #[serde(default)]
    pub dir: Option<String>,
    pub parent: Option<String>,
    pub version: Option<String>,
    pub commands: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct LayerSpec {
    #[serde(default)]
    pub commands: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct BuildSpec {
    // deprecated
    #[doku(example = "Inline")]
    pub kind: BuildKind,

    // deprecated
    #[serde(default)]
    #[doku(example = "dnf install git -yy")]
    pub pre: Vec<String>,

    #[serde(default)]
    pub post: Vec<String>,

    #[serde(default)]
    pub package_manager: Option<String>,

    #[serde(default)]
    pub force: Option<bool>,

    /// Command to use when build kind is Code
    #[serde(default = "default_command")]
    #[doku(example = "zip -9 lambda.zip .")]
    pub command: String,

    #[serde(default)]
    pub images: HashMap<String, ImageSpec>,

    #[serde(default)]
    pub layers: HashMap<String, LayerSpec>,
}

impl BuildSpec {
    pub fn new(dir: &str) -> BuildSpec {
        let path = format!("{}/build.json", dir);
        let data = u::slurp(&path);
        let bspec: BuildSpec = serde_json::from_str(&data).unwrap();
        bspec
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub enum Platform {
    Lambda,
    Fargate,
}

impl FromStr for Platform {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lambda" | "Lambda"  => Ok(Platform::Lambda),
            "farget" | "Fargate" => Ok(Platform::Fargate),
            _                    => Ok(Platform::Lambda),
        }
    }
}

impl Platform {
    pub fn to_str(&self) -> String {
        match self {
            Platform::Lambda => s!("lambda"),
            Platform::Fargate => s!("fargate"),
        }
    }
}

fn default_platform() -> Option<Platform> {
    Some(Platform::Lambda)
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct RuntimeSpec {
    #[serde(default = "default_lang")]
    pub lang: LangRuntime,

    #[serde(default = "default_handler")]
    pub handler: String,

    #[serde(default = "default_package_type")]
    pub package_type: String,

    #[serde(default = "default_platform")]
    pub platform: Option<Platform>,

    pub vars_file: Option<String>,
    pub role_file: Option<String>,
    pub role: Option<String>,

    pub uri: Option<String>,

    pub mount_fs: Option<bool>,

    pub snapstart: Option<bool>,

    #[serde(default = "default_layers")]
    pub layers: Vec<String>,

    #[serde(default = "default_layers")]
    pub extensions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct Role {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct InfraSpec {
    #[serde(default = "default_infra_dir")]
    pub dir: String,

    #[serde(default)]
    pub vars_file: Option<String>,

    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
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

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct FunctionSpec {
    pub name: String,
    pub dir: Option<String>,
    pub description: Option<String>,
    pub namespace: Option<String>,
    pub fqn: Option<String>,
    pub layer_name: Option<String>,
    pub version: Option<String>,
    pub revision: Option<String>,
    pub runtime: Option<RuntimeSpec>,
    pub build: Option<BuildSpec>,
    pub infra: Option<InfraSpec>,
    //deprecated
    pub infra_dir: Option<String>,
    //deprecated
    #[serde(default)]
    pub tasks: HashMap<String, String>,
    //deprecated
    pub assets: Option<AssetsSpec>,
}

fn find_revision(dir: &str) -> String {
    let cmd_str = format!("git log -n 1 --format=%h {}", dir);
    u::sh(&cmd_str, dir)
}

fn top_level() -> String {
    u::sh("git rev-parse --show-toplevel", &u::pwd())
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

impl FunctionSpec {
    pub fn new(dir: &str) -> FunctionSpec {
        let f = format!("{}/function.json", dir);
        let version = find_revision(dir);
        if u::file_exists(&f) {
            let data = render(&u::slurp(&f), &version);
            let fspec = serde_json::from_str(&data);
            match fspec {
                Ok(f) => f,
                Err(e) => panic!("{}", e),
            }
        } else {
            FunctionSpec {
                name: u::basedir(dir).to_string(),
                dir: Some(dir.to_string()),
                description: None,
                namespace: None,
                fqn: None,
                layer_name: None,
                version: None,
                revision: None,
                runtime: None,
                build: None,
                infra: None,
                infra_dir: None,
                assets: None,
                tasks: HashMap::new(),
            }
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOutput {
    pub name: String,
    pub dir: String,
    pub runtime: LangRuntime,
    pub kind: BuildKind,
    pub artifact: String,
}
