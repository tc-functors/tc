use crate::{
    spec::{
        RouteSpec,
        TopologySpec,
        Entity,
        config::ConfigSpec,
    },
};
use super::template;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub gateway: String,
    pub authorizer: String,
    pub entity: Entity,
    pub target_arn: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>,
    pub sync: bool,
    pub request_template: String,
    pub response_template: String,
}

fn make_response_template() -> String {
    format!(r#"#set ($parsedPayload = $util.parseJson($input.json('$.output'))) $parsedPayload"#)
}

fn make_request_template(method: &str, request_template: Option<String>) -> String {
    if method == "POST" {
        match request_template {
            Some(r) => match r.as_ref() {
                "detail" => s!("\"{\"path\": \"${request.path}\", \"detail\": ${request.body.detail}, \"method\": \"${context.httpMethod}\"}\""),
                "merged" => s!("\"{\"path\": $request.path, \"body\": $request.body}\""),
                _ => r
            },
            None => s!("${request.body}")
        }
    } else {
        s!("\"{\"path\": \"${request.path}\", \"method\": \"${context.httpMethod}\"}\"")
    }
}

fn find_target_arn(target_name: &str, entity: &Entity) -> String {
    match entity {
        Entity::Function => template::api_integration_arn(&target_name),
        Entity::State => template::sfn_arn(&target_name),
        _ => template::lambda_arn(&target_name),
    }
}

impl Route {
    pub fn new(
        name: &str,
        spec: &TopologySpec,
        rspec: &RouteSpec,
        config: &ConfigSpec
) -> Route {

        let gateway =  match &rspec.gateway {
            Some(gw) => gw.clone(),
            None => config.aws.api_gateway.api_name.clone()
        };

        let path = match &rspec.path {
            Some(p) => p.clone(),
            None => s!(name)
        };

        let method = match &rspec.method {
            Some(m) => m.clone(),
            None => s!("POST")
        };

        let entity = match &rspec.proxy {
            Some(_) => Entity::Function,
            None => match rspec.function {
                Some(_) => Entity::Function,
                None => Entity::State,
            },
        };

        let target_name = match &rspec.proxy {
            Some(f) => s!(f),
            None => match &rspec.function {
                Some(x) => s!(x),
                None => template::topology_fqn(&spec.name, spec.hyphenated_names),
            },
        };

        let target_arn = find_target_arn(&target_name, &entity);

        let sync = match rspec.sync {
            Some(s) => s,
            None => false
        };

        let stage = match &rspec.stage {
            Some(s) => s.clone(),
            None => s!("$default")
        };

        Route {
            method: method.clone(),
            path: path,
            gateway: gateway,
            authorizer: rspec.authorizer.clone(),
            entity: entity,
            target_arn: target_arn,
            stage: stage,
            stage_variables: HashMap::new(),
            sync: sync,
            request_template: make_request_template(&method, rspec.request_template.clone()),
            response_template: make_response_template(),
        }
    }
}
