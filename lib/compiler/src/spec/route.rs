use serde_derive::{
    Deserialize,
    Serialize,
};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorsSpec {
    pub methods: Vec<String>,
    pub origins: Vec<String>
}

fn default_cors() -> Option<CorsSpec> {
    let c = CorsSpec {
        methods: vec![String::from("GET"), String::from("POST")],
        origins: vec![String::from("*")],
    };
    Some(c)
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
    #[serde(default = "default_cors")]
    pub cors: Option<CorsSpec>
}
