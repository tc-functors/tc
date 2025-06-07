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

use std::path::PathBuf;

pub mod function;
pub mod config;
pub mod infra;
pub mod event;
pub mod route;
pub mod mutation;
pub mod queue;
pub mod channel;

use crate::parser;

use parser::yaml::Transformer;

pub use function::{
    FunctionSpec, BuildOutput, BuildKind,
    LangRuntime,
    Lang,
    BuildSpec,
    ImageSpec
};
pub use config::ConfigSpec;
pub use event::EventSpec;
pub use route::RouteSpec;
pub use queue::QueueSpec;
pub use channel::ChannelSpec;
pub use mutation::MutationSpec;

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
    Mutation,
    #[serde(alias = "trigger")]
    Trigger
}

impl FromStr for Entity {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "function" | "functions" => Ok(Entity::Function),
            "queue"    | "queues" => Ok(Entity::Queue),
            "route"    | "routes" => Ok(Entity::Route),
            "channel"  | "channels" => Ok(Entity::Channel),
            "event"    | "events" => Ok(Entity::Event),
            "state"    | "states" => Ok(Entity::State),
            "mutation" | "mutations" => Ok(Entity::Mutation),
            "trigger" | "triggers" => Ok(Entity::Trigger),
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
            Entity::State    => s!("state"),
            Entity::Trigger  => s!("trigger")
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

fn default_functions() -> Functions {
    Functions { shared: vec![] }
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

    #[serde(default)]
    pub pools: Option<Vec<String>>,

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
            let path = PathBuf::from(topology_spec_file);

            match std::env::var("TC_SPEC_SIMPLE") {
                Ok(_) => {
                    let data: String = u::slurp(topology_spec_file);
                    let spec: TopologySpec = serde_yaml::from_str(&data).unwrap();
                    spec
                },
                Err(_) => {
                    let tn = Transformer::new(path, false);
                    let v = match tn {
                        Ok(transformer) => transformer.parse(),
                        Err(e) => panic!("{:?}", e)
                    };
                    let spec: TopologySpec = serde_yaml::from_value(v).unwrap();
                    spec
                }
            }

        } else {
            TopologySpec {
                name: s!("tc"),
                kind: Some(TopologyKind::Function),
                hyphenated_names: false,
                version: None,
                infra: None,
                config: None,
                mode: None,
                pools: None,
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
