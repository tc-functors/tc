use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use super::spec::RouteSpec;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub kind: String,
    pub method: String,
    pub path: String,
    pub gateway: String,
    pub authorizer: String,
    pub proxy: String,
    pub stage: Option<String>,
    pub stage_variables: HashMap<String, String>,
}

impl Route {

    pub fn new(spec: &RouteSpec) -> Route {
        Route {
            kind: spec.kind.clone(),
            method: spec.method.clone(),
            path: spec.path.clone(),
            gateway: spec.gateway.clone(),
            authorizer: spec.authorizer.clone(),
            proxy: spec.proxy.clone(),
            stage: None,
            stage_variables: HashMap::new()

        }
    }

}
