use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::PathBuf,
};

pub mod channel;
pub mod config;
pub mod event;
pub mod function;
pub mod infra;
pub mod mutation;
pub mod page;
pub mod queue;
pub mod route;

use crate::parser;
pub use channel::ChannelSpec;
pub use config::ConfigSpec;
pub use event::EventSpec;
pub use function::{
    BuildKind,
    BuildSpec,
    FunctionSpec,
    ImageSpec,
    InlineFunctionSpec,
    Lang,
    LangRuntime,
    TestSpec
};
pub use mutation::MutationSpec;
pub use page::PageSpec;
use parser::yaml::Transformer;
pub use queue::QueueSpec;
pub use route::RouteSpec;

// topology


fn default_nodes() -> Nodes {
    Nodes {
        root: Some(false),
        ignore: Some(vec![]),
        dirs: Some(vec![]),
    }
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

    pub recursive: Option<bool>,
    pub auto: Option<bool>,

    pub dir: Option<String>,

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
    pub functions: Option<HashMap<String, InlineFunctionSpec>>,
    pub events: Option<HashMap<String, EventSpec>>,
    pub routes: Option<HashMap<String, RouteSpec>>,
    pub mutations: Option<MutationSpec>,
    pub queues: Option<HashMap<String, QueueSpec>>,
    pub channels: Option<HashMap<String, ChannelSpec>>,
    pub triggers: Option<HashMap<String, TriggerSpec>>,
    pub pages: Option<HashMap<String, PageSpec>>,
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
                    let mut spec: TopologySpec = serde_yaml::from_str(&data).unwrap();
                    spec.dir = Some(u::parent_dir(topology_spec_file));
                    spec
                }
                Err(_) => {
                    let tn = Transformer::new(path, false);
                    let v = match tn {
                        Ok(transformer) => transformer.parse(),
                        Err(e) => panic!("{:?}", e),
                    };
                    let mut spec: TopologySpec = serde_yaml::from_value(v).unwrap();
                    spec.dir = Some(u::parent_dir(topology_spec_file));
                    spec
                }
            }
        } else {
            TopologySpec {
                name: s!("tc"),
                recursive: Some(false),
                auto: Some(false),
                kind: Some(TopologyKind::Function),
                dir: Some(u::pwd()),
                hyphenated_names: false,
                version: None,
                infra: None,
                config: None,
                mode: None,
                pools: None,
                functions: None,
                routes: None,
                events: None,
                nodes: default_nodes(),
                states: None,
                flow: None,
                queues: None,
                mutations: None,
                channels: None,
                triggers: None,
                pages: None,
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
