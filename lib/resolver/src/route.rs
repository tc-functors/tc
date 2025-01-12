use super::{Context, Topology};
use kit::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub kind: String,
    pub method: String,
    pub path: String,
    pub gateway: String,
    pub authorizer: String,
    pub proxy: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>,
}

fn is_stable(sandbox: &str) -> bool {
    sandbox == "stable"
}

pub fn make(context: &Context, topology: &Topology) -> HashMap<String, Route> {
    let mut routes: HashMap<String, Route> = HashMap::new();
    let sandbox = &context.sandbox;

    let stage = if is_stable(&context.sandbox) {
        s!("$default")
    } else {
        sandbox.to_string()
    };
    let mut stage_variables: HashMap<String, String> = HashMap::new();
    stage_variables.insert(s!("sandbox"), sandbox.to_string());

    if let Some(r) = &topology.routes {
        for (id, route) in r {
            let path = if is_stable(&sandbox) {
                route.path.to_owned()
            } else {
                format!("/{}{}", sandbox, &route.path)
            };

            let r = Route {
                kind: route.kind.to_owned(),
                method: route.method.to_owned(),
                path: path,
                gateway: context.render(&route.gateway),
                authorizer: context.render(&route.authorizer),
                proxy: context.render(&route.proxy),
                stage: stage.clone(),
                stage_variables: stage_variables.clone(),
            };
            routes.insert(id.to_string(), r);
        }
    }
    routes
}
