use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub kind: String,
    pub method: String,
    pub path: String,
    pub gateway: String,
    pub authorizer: String,
    pub proxy: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>
}
