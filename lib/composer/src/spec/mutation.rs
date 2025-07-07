use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MutationConsumer {
    pub name: String,
    pub mapping: HashMap<String, String>,
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
