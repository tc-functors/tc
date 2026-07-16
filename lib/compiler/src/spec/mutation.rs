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
    pub authorizer: Option<String>,
    #[serde(default)]
    pub inputs: Option<HashMap<String, HashMap<String, String>>>,
    #[serde(default)]
    pub types: HashMap<String, HashMap<String, String>>,
    pub resolvers: HashMap<String, ResolverSpec>,
}


pub fn merge_specs(mspecs: &Vec<MutationSpec>) -> MutationSpec {
    let mut inputs: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut types: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut resolvers: HashMap<String, ResolverSpec> = HashMap::new();
    let mut authorizer: Option<String> = None;
    for spec in mspecs {
        if let Some(ins) = spec.inputs.clone() {
            inputs.extend(ins);
        }
        types.extend(spec.types.clone());
        resolvers.extend(spec.resolvers.clone());
        authorizer = spec.authorizer.clone();
    }
    MutationSpec {
        authorizer: authorizer,
        inputs: Some(inputs),
        types: types,
        resolvers: resolvers
    }
}
