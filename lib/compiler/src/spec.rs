use doku::Document;
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    str::FromStr,
};

pub mod function;
pub mod config;
pub mod infra;

pub use function::{
    FunctionSpec, BuildOutput, BuildKind,
    LangRuntime,
    Lang,
    BuildSpec,
    ImageSpec
};
pub use config::ConfigSpec;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Document)]
pub enum Entity {
    #[serde(alias = "function")]
    Function,
    #[serde(alias = "queue")]
    Queue,
    #[serde(alias = "route")]
    Route,
    #[serde(alias = "channel")]
    Channel,
    #[serde(alias = "event")]
    Event,
    #[serde(alias = "state")]
    State,
    #[serde(alias = "mutation")]
    Mutation
}

impl FromStr for Entity {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "function" => Ok(Entity::Function),
            "queue"    => Ok(Entity::Queue),
            "route"    => Ok(Entity::Route),
            "channel"  => Ok(Entity::Channel),
            "event"    => Ok(Entity::Event),
            "state"    => Ok(Entity::State),
            "mutation" => Ok(Entity::Mutation),
            _          => Ok(Entity::Function),
        }
    }
}

impl Entity {

    pub fn to_str(&self) -> String {
        match self {
            Entity::Function => s!("function"),
            Entity::Queue    => s!("queue"),
            Entity::Route    => s!("route"),
            Entity::Channel  => s!("channel"),
            Entity::Event    => s!("event"),
            Entity::Mutation => s!("mutation"),
            Entity::State    => s!("state")
        }
    }
}

// topology

fn default_nodes() -> Nodes {
    Nodes {
        root: Some(false),
        ignore: Some(vec![]),
        dirs: Some(vec![]),
    }
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
pub struct EventSpec {
    #[serde(default)]
    pub producer: String,

    #[serde(default)]
    pub doc_only: bool,

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
    pub channel: Option<String>,

    #[serde(default)]
    pub stepfunction: Option<String>,

    #[serde(default)]
    pub pattern: Option<String>,

    #[serde(default)]
    pub sandboxes: Vec<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HandlerSpec {
    #[serde(default)]
    pub handler: Option<String>,

    #[serde(default)]
    pub event: Option<String>,

    #[serde(default)]
    pub function: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelSpec {
    #[serde(default)]
    pub doc_only: bool,
    pub on_publish: Option<HandlerSpec>,
    pub on_subscribe: Option<HandlerSpec>,
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
    pub method: Option<String>,
    pub path: Option<String>,
    pub gateway: Option<String>,

    #[serde(default)]
    pub authorizer: String,

    pub proxy: Option<String>,
    pub function: Option<String>,
    pub state: Option<String>,
    pub event: Option<String>,
    pub queue: Option<String>,

    pub request_template: Option<String>,
    pub response_template: Option<String>,
    pub sync: Option<bool>,

    pub stage: Option<String>,
    pub stage_variables: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Nodes {
    #[serde(default)]
    pub ignore: Option<Vec<String>>,
    #[serde(default)]
    pub root: Option<bool>,
    #[serde(default)]
    pub dirs: Option<Vec<String>>,
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
pub struct TriggerSpec {
    #[serde(default)]
    pub function: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TopologyKind {
    #[serde(alias = "step-function", alias = "state-machine")]
    StepFunction,
    #[serde(alias = "function")]
    Function,
    #[serde(alias = "evented")]
    Evented,
    #[serde(alias = "grapqhl")]
    Graphql,
}

impl TopologyKind {
    pub fn to_str(&self) -> String {
        match self {
            TopologyKind::StepFunction => s!("stepfn"),
            TopologyKind::Function => s!("function"),
            TopologyKind::Graphql => s!("graphql"),
            TopologyKind::Evented => s!("evented"),
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

    #[serde(default)]
    pub config: Option<String>,

    pub mode: Option<String>,

    #[serde(default)]
    pub hyphenated_names: bool,

    #[serde(default = "default_nodes")]
    pub nodes: Nodes,
    #[serde(default = "default_functions")]
    pub functions: Functions,
    pub events: Option<HashMap<String, EventSpec>>,
    pub routes: Option<HashMap<String, RouteSpec>>,
    pub mutations: Option<MutationSpec>,
    pub queues: Option<HashMap<String, QueueSpec>>,
    pub channels: Option<HashMap<String, ChannelSpec>>,
    pub triggers: Option<HashMap<String, TriggerSpec>>,
    pub states: Option<Value>,
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
                config: None,
                mode: None,
                functions: Functions { shared: vec![] },
                routes: None,
                events: None,
                nodes: default_nodes(),
                states: None,
                flow: None,
                queues: None,
                mutations: None,
                channels: None,
                triggers: None
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
