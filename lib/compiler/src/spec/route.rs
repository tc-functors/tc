use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorsSpec {
    pub methods: Vec<String>,
    pub origins: Vec<String>,
    #[serde(alias = "headers", alias = "allowed_headers")]
    pub headers: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteSpec {
    pub method: Option<String>,
    pub path: Option<String>,
    pub gateway: Option<String>,
    pub authorizer: Option<String>,
    #[serde(default)]
    pub function: Option<String>,
    #[serde(default)]
    pub proxy: Option<String>,
    pub state: Option<String>,
    pub event: Option<String>,
    pub queue: Option<String>,

    pub request_template: Option<String>,
    pub response_template: Option<String>,
    #[serde(alias = "async")]
    pub is_async: Option<bool>,

    pub stage: Option<String>,
    pub stage_variables: Option<HashMap<String, String>>,
    pub cors: Option<CorsSpec>,
    #[serde(default, alias = "doc-only")]
    pub doc_only: bool,
}
