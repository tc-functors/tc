use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use kit::*;
use kit as u;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Lang {
    Python,
    Ruby,
    Go,
    Rust,
    Node,
    Clojure
}

impl FromStr for Lang {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "python3.10" | "python3.11" | "python3.9" | "python3.12" => Ok(Lang::Python),
            "ruby3.2" | "ruby" | "ruby32"             => Ok(Lang::Ruby),
            "node22" | "node20" | "node18"            => Ok(Lang::Node),
            "rust"                                    => Ok(Lang::Rust),
            _                                         => Ok(Lang::Python)
        }
    }
}


#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum LangRuntime {
    #[serde(alias="python3.9")]
    Python39,
    #[serde(alias="python3.10")]
    Python310,
    #[serde(alias="python3.11")]
    Python311,
    #[serde(alias="python3.12")]
    Python312,
    #[serde(alias="python3.13")]
    Python313,
    #[serde(alias="ruby3.2")]
    Ruby32,
    #[serde(alias="java21")]
    Java21,
    #[serde(alias="rust")]
    Rust,
    #[serde(alias="node22")]
    Node22,
    #[serde(alias="node20")]
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
            _                             => Ok(LangRuntime::Python311)
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BuildKind {
    Code,
    Inline,
    Layer,
    Library,
    Extension,
    Runtime,
    Image
}

impl FromStr for BuildKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code"      => Ok(BuildKind::Code),
            "inline"    => Ok(BuildKind::Inline),
            "layer"     => Ok(BuildKind::Layer),
            "library"   => Ok(BuildKind::Library),
            "extension" => Ok(BuildKind::Extension),
            "runtime"   => Ok(BuildKind::Runtime),
            "image"     => Ok(BuildKind::Image),
            _           => Ok(BuildKind::Layer)
        }
    }
}

impl BuildKind {

    pub fn to_str(&self) -> String {
        match self {
            BuildKind::Code      => s!("code"),
            BuildKind::Inline    => s!("inline"),
            BuildKind::Layer     => s!("layer"),
            BuildKind::Library   => s!("library"),
            BuildKind::Extension => s!("extension"),
            BuildKind::Runtime   => s!("runtime"),
            BuildKind::Image     => s!("image")
        }

    }
}

// function infra spec


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeNetworkSpec {
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeFilesystemSpec {
    pub arn: String,
    pub mount_point: String,
}

fn default_memory_size() -> Option<i32> {
    Some(128)
}


fn default_timeout() -> Option<i32> {
    Some(300)
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeInfraSpec {
    #[serde(default = "default_memory_size")]
    pub memory_size: Option<i32>,
    #[serde(default = "default_timeout")]
    pub timeout: Option<i32>,
    pub image_uri: Option<String>,
    pub provisioned_concurrency: Option<i32>,
    pub environment: Option<HashMap<String, String>>,
    pub network: Option<RuntimeNetworkSpec>,
    pub filesystem: Option<RuntimeFilesystemSpec>,
    pub tags: Option<HashMap<String, String>>,
}

impl RuntimeInfraSpec {

    pub fn new(runtime_file: Option<String>) -> HashMap<String, RuntimeInfraSpec> {
        match runtime_file {
            Some(f) => {
                let data = u::slurp(&f);
                let ris: HashMap<String, RuntimeInfraSpec> = serde_json::from_str(&data).unwrap();
                ris
            }
            None => {
                let mut h: HashMap<String, RuntimeInfraSpec> = HashMap::new();
                let r = RuntimeInfraSpec {
                    memory_size: Some(128),
                    timeout: Some(300),
                    image_uri: None,
                    provisioned_concurrency: None,
                    environment: None,
                    network: None,
                    filesystem: None,
                    tags: None
                };
                h.insert(s!("default"), r);
                h
            }
        }
    }
}

// function

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildSpec {
    pub kind: BuildKind,

    #[serde(default)]
    pub pre: Vec<String>,

    #[serde(default)]
    pub post: Vec<String>,

    #[serde(default = "default_command")]
    pub command: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeSpec {
    #[serde(default = "default_lang")]
    pub lang: LangRuntime,

    #[serde(default = "default_handler")]
    pub handler: String,

    #[serde(default = "default_package_type")]
    pub package_type: String,

    pub vars_file: Option<String>,
    pub role_file: Option<String>,

    pub uri: Option<String>,

    pub mount_fs: Option<bool>,

    pub snapstart: Option<bool>,

    #[serde(default = "default_layers")]
    pub layers: Vec<String>,

    #[serde(default = "default_layers")]
    pub extensions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InfraSpec {
    #[serde(default = "default_infra_dir")]
    pub dir: String,

    #[serde(default)]
    pub vars_file: Option<String>,

    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    #[serde(default)]
    pub assets: HashMap<String, Value>,

}

fn find_revision(dir: &str) -> String {
    let cmd_str = format!("git log -n 1 --format=%h {}", dir);
    u::sh(&cmd_str, dir)
}

fn render(s: &str, version: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("version", version);
    table.insert("sandbox", "{{sandbox}}");
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
                Err(e) => panic!("{}", e)
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
                assets: HashMap::new(),
                tasks: HashMap::new()
            }
        }
    }
}


// topology

fn default_nodes() -> Nodes {
    Nodes { ignore: vec![], dirs: vec![] }
}

fn default_route_kind() -> String {
    s!("http")
}

fn default_target() -> String {
    s!("")
}

fn default_function() -> Option<String> {
    None
}

fn default_source() -> Vec<String> {
    vec![]
}

fn default_targets() -> Vec<String> {
    vec![]
}

fn default_functions() -> Functions {
    Functions { shared: vec![] }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MutationConsumer {
    pub name: String,
    pub mapping: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Produces {
    pub consumer: String,

    #[serde(default = "default_source")]
    pub source: Vec<String>,

    #[serde(default)]
    pub filter: Option<String>,

    #[serde(default = "default_target")]
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Consumes {
    #[serde(default)]
    pub producer: String,

    pub producer_ns: Option<String>,

    pub nth: Option<u8>,

    #[serde(default)]
    pub filter: Option<String>,

    #[serde(default)]
    pub rule_name: Option<String>,

    #[serde(default = "default_function")]
    pub function: Option<String>,

    #[serde(default = "default_targets")]
    pub functions: Vec<String>,

    #[serde(default)]
    pub mutation: Option<String>,

    #[serde(default)]
    pub stepfunction: Option<String>,

    #[serde(default)]
    pub pattern: Option<String>,

    #[serde(default)]
    pub sandboxes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventsSpec {
    #[serde(default)]
    pub doc_only: bool,
    pub consumes: Option<HashMap<String, Consumes>>,
    pub produces: Option<HashMap<String, Produces>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueueSpec {
    #[serde(default)]
    pub producer: String,

    #[serde(default)]
    pub consumer: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteSpec {
    #[serde(default = "default_route_kind")]
    pub kind: String,
    pub method: String,
    pub path: String,
    pub gateway: String,

    #[serde(default)]
    pub authorizer: String,

    pub proxy: Option<String>,
    pub function: Option<String>,
    pub stage: Option<String>,
    pub stage_variables: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Nodes {
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub dirs: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Functions {
    pub shared: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResolverSpec {
    pub input: String,

    pub output: String,

    #[serde(default)]
    pub function: Option<String>,

    #[serde(default)]
    pub event: Option<String>,

    #[serde(default)]
    pub table: Option<String>,

    pub subscribe: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MutationSpec {
    #[serde(default)]
    pub authorizer: String,

    #[serde(default)]
    pub types: HashMap<String, HashMap<String, String>>,
    pub resolvers: HashMap<String, ResolverSpec>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScheduleSpec {
    pub cron: String,
    pub target: String,
    pub payload: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TopologyKind {
    #[serde(alias="step-function",alias="state-machine")]
    StepFunction,
    #[serde(alias="function")]
    Function,
    #[serde(alias="evented")]
    Evented,
    #[serde(alias="grapqhl")]
    Graphql
}

impl TopologyKind {

    pub fn to_str(&self) -> String {
        match self {
            TopologyKind::StepFunction => s!("stepfn"),
            TopologyKind::Function => s!("function"),
            TopologyKind::Graphql => s!("graphql"),
            TopologyKind::Evented => s!("evented")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TopologySpec {
    #[serde(default)]
    pub name: String,

    pub kind: Option<TopologyKind>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub infra: Option<String>,

    pub mode: Option<String>,

    #[serde(default)]
    pub hyphenated_names: bool,

    #[serde(default = "default_nodes")]
    pub nodes: Nodes,
    #[serde(default = "default_functions")]
    pub functions: Functions,
    pub events: Option<EventsSpec>,
    pub routes: Option<HashMap<String, RouteSpec>>,
    pub states: Option<Value>,
    pub mutations: Option<MutationSpec>,
    pub queues: Option<HashMap<String, QueueSpec>>,
    pub flow: Option<Value>,
}

impl TopologySpec {

    pub fn new(topology_spec_file: &str) -> TopologySpec {
        if u::file_exists(topology_spec_file) {
            tracing::debug!("Loading topology {}", topology_spec_file);
            let data: String = u::slurp(topology_spec_file);
            let spec: TopologySpec = serde_yaml::from_str(&data).unwrap();
            spec

        } else {
            TopologySpec {
                name: s!("tc"),
                kind: Some(TopologyKind::Function),
                hyphenated_names: false,
                version: None,
                infra: None,
                mode: None,
                functions: Functions { shared: vec![] },
                routes: None,
                events: None,
                nodes: Nodes { ignore: vec![], dirs: vec![] },
                states: None,
                flow: None,
                queues: None,
                mutations: None,
            }
        }
    }

    pub fn fmt(&self) -> &str {
        if self.hyphenated_names {
            "hyphenated"
        } else {
            "regular"
        }
    }

}

// component
pub enum Component {
    Function,
    Event,
    Mutation,
    Route,
    Queue,
    Node,
    Flow
}
