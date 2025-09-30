use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use safe_unwrap::safe_unwrap;
use std::{
    collections::HashMap,
    path::PathBuf,
};

pub mod channel;
pub mod event;
pub mod function;
pub mod mutation;
pub mod page;
pub mod queue;
pub mod route;
pub mod table;
pub mod test;
pub mod tag;
pub mod role;
pub mod schedule;
pub mod template;
pub mod state;

use crate::walker;
use crate::yaml;
pub use channel::ChannelSpec;
pub use event::EventSpec;
pub use function::{
    build::BuildKind,
    build::BuildSpec,
    runtime::RuntimeSpec,
    FunctionSpec,
    runtime::Lang,
    runtime::LangRuntime,
};
pub use mutation::MutationSpec;
pub use page::PageSpec;
pub use queue::QueueSpec;
pub use route::RouteSpec;
pub use table::TableSpec;
pub use test::TestSpec;
pub use role::RoleSpec;
pub use schedule::ScheduleSpec;
use yaml::Transformer;
use configurator::Config;

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
    #[serde(alias = "routed")]
    Routed,
}

impl TopologyKind {
    pub fn to_str(&self) -> String {
        match self {
            TopologyKind::StepFunction => s!("stepfn"),
            TopologyKind::Function => s!("function"),
            TopologyKind::Graphql => s!("graphql"),
            TopologyKind::Evented => s!("evented"),
            TopologyKind::Routed => s!("routed"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TopologySpec {
    #[serde(default)]
    pub name: String,
    pub recursive: Option<bool>,
    pub auto: Option<bool>,
    pub fqn: Option<String>,
    pub dir: Option<String>,

    pub kind: Option<TopologyKind>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub infra: Option<String>,

    pub config: Option<Config>,

    pub mode: Option<String>,

    #[serde(default)]
    pub pools: Option<Vec<String>>,

    #[serde(default = "default_nodes")]
    pub nodes: Nodes,
    pub children: Option<HashMap<String, TopologySpec>>,
    pub functions: Option<HashMap<String, FunctionSpec>>,
    pub events: Option<HashMap<String, EventSpec>>,
    pub routes: Option<HashMap<String, RouteSpec>>,
    pub mutations: Option<MutationSpec>,
    pub queues: Option<HashMap<String, QueueSpec>>,
    pub channels: Option<HashMap<String, ChannelSpec>>,
    pub triggers: Option<HashMap<String, TriggerSpec>>,
    pub pages: Option<HashMap<String, PageSpec>>,
    pub tables: Option<HashMap<String, TableSpec>>,
    pub schedules: Option<HashMap<String, ScheduleSpec>>,
    pub roles: Option<HashMap<String, RoleSpec>>,
    pub tests: Option<HashMap<String, TestSpec>>,
    pub tags: Option<HashMap<String, String>>,
    pub states: Option<Value>,
    pub flow: Option<Value>,
}

impl Default for TopologySpec {
    fn default() -> Self {

        let config = Config::new();
        TopologySpec {
            name: s!("tc"),
            recursive: Some(false),
            auto: Some(false),
            fqn: None,
            kind: Some(TopologyKind::Function),
            dir: Some(u::pwd()),
            version: None,
            children: None,
            infra: None,
            mode: None,
            pools: None,
            functions: None,
            routes: None,
            events: None,
            nodes: default_nodes(),
            states: None,
            roles: None,
            flow: None,
            queues: None,
            mutations: None,
            channels: None,
            schedules: None,
            triggers: None,
            tables: None,
            pages: None,
            tags: None,
            tests: None,
            config: Some(config)
        }
    }

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TopologyMetadata {
    pub name: String,
    pub fqn: String,
    pub dir: String,
    pub kind: TopologyKind,
    pub version: String,
    pub infra: String,
    pub config: Config,
}

impl TopologySpec {

    pub fn new(topology_spec_file: &str) -> TopologySpec {
        if u::file_exists(topology_spec_file) {
            tracing::debug!("Loading topology {}", topology_spec_file);
            let path = PathBuf::from(topology_spec_file);

            let tn = Transformer::new(path, false);
            let v = match tn {
                Ok(transformer) => transformer.parse(),
                Err(e) => panic!("{:?}", e),
            };
            let mut spec: TopologySpec = serde_yaml::from_value(v).unwrap();
            let dir = u::parent_dir(topology_spec_file);
            spec.states = state::make(&dir, &spec);
            spec.dir = Some(dir);
            spec
        } else {
            TopologySpec::default()
        }
    }

    pub fn standalone(dir: &str, namespace: &str, functions: HashMap<String, FunctionSpec>) -> TopologySpec {
        let config = Config::new();
        TopologySpec {
            name: s!(namespace),
            dir: Some(s!(dir)),
            kind: Some(TopologyKind::Function),
            functions: Some(functions),
            config: Some(config),
            ..Default::default()
        }
    }

    pub fn walk(&self) -> TopologySpec {
        walker::walk(&self)
    }

    pub fn to_yaml(&self) {
        let yaml = serde_yaml::to_string(self).unwrap();
        println!("{}", &yaml);
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn to_bincode(&self) {
        let byea: Vec<u8> = bincode::serialize(self).unwrap();
        let path = format!("{}.tc", self.name);
        kit::write_bytes(&path, byea);
    }

    pub fn read_bincode(path: &str) -> TopologySpec {
        let data = kit::read_bytes(path);
        let t: TopologySpec = bincode::deserialize(&data).unwrap();
        t
    }

    pub fn metadata(&self) -> TopologyMetadata {
        TopologyMetadata {
            name: self.name.clone(),
            dir: safe_unwrap!("No dir found", self.dir.clone()),
            fqn: safe_unwrap!("No fqn found", self.fqn.clone()),
            kind: safe_unwrap!("No kind found", self.kind.clone()),
            version: safe_unwrap!("No version found", self.version.clone()),
            infra: safe_unwrap!("No infra found", self.infra.clone()),
            config: safe_unwrap!("No config found", self.config.clone())
        }
    }

}
