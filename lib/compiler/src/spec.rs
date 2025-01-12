use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use kit as u;
use kit::*;

fn default_functions() -> Functions {
    Functions { shared: vec![] }
}

fn default_nodes() -> Nodes {
    Nodes { ignore: vec![] }
}

fn default_route_kind() -> String {
    s!("http")
}

fn default_kind() -> String {
    s!("step-function")
}

fn default_proxy() -> String {
    s!("none")
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicSpec {
    pub name: String,
    #[serde(default)]
    pub hyphenated_names: bool,
}

impl BasicSpec {
    pub fn new(topology_spec_file: &str) -> BasicSpec {
        if u::file_exists(topology_spec_file) && is_valid_spec(topology_spec_file) {
            let data: String = u::slurp(topology_spec_file);
            let spec: BasicSpec = serde_yaml::from_str(&data).unwrap();
            spec
        } else {
            BasicSpec {
                name: String::from("tc"),
                hyphenated_names: false,
            }
        }
    }
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
pub struct MutationConsumer {
    pub name: String,
    pub mapping: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Consumes {
    #[serde(default)]
    pub producer: String,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default = "default_function")]
    pub function: Option<String>,
    #[serde(default)]
    pub mutation: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub sandboxes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Events {
    pub consumes: Option<HashMap<String, Consumes>>,
    pub produces: Option<HashMap<String, Produces>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Queue {
    #[serde(default)]
    pub producer: String,
    #[serde(default)]
    pub consumer: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    #[serde(default = "default_route_kind")]
    pub kind: String,
    pub method: String,
    pub path: String,
    pub gateway: String,
    #[serde(default)]
    pub authorizer: String,
    #[serde(default = "default_proxy")]
    pub proxy: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Nodes {
    pub ignore: Vec<String>,
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
pub struct Spec {
    pub name: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub infra: Option<String>,
    pub mode: Option<String>,
    #[serde(default)]
    pub hyphenated_names: bool,
    #[serde(default = "default_functions")]
    pub functions: Functions,
    #[serde(default = "default_nodes")]
    pub nodes: Nodes,
    pub events: Option<Events>,
    pub routes: Option<HashMap<String, Route>>,
    pub states: Option<Value>,
    pub mutations: Option<MutationSpec>,
    #[serde(default)]
    pub queues: HashMap<String, Queue>,
    pub flow: Option<Value>,
}

fn is_valid_spec(f: &str) -> bool {
    u::file_size(f) != 0.0
}

impl Spec {
    pub fn new(topology_spec_file: &str) -> Spec {
        if u::file_exists(topology_spec_file) && is_valid_spec(topology_spec_file) {
            let data: String = u::slurp(topology_spec_file);
            let spec: Spec = serde_yaml::from_str(&data).unwrap();
            spec
        } else {
            Spec {
                name: s!("tc"),
                kind: s!("step-function"),
                hyphenated_names: false,
                version: None,
                infra: None,
                mode: None,
                functions: Functions { shared: vec![] },
                routes: None,
                events: None,
                nodes: Nodes { ignore: vec![] },
                states: None,
                flow: None,
                queues: HashMap::new(),
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
