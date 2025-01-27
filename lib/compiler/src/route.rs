use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use kit::*;
use super::template;
use super::spec::RouteSpec;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TargetKind {
    Function,
    StepFunction
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RouteKind {
    Function,
    StepFunction
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub gateway: String,
    pub authorizer: String,
    pub target_kind: TargetKind,
    pub target_arn: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>,
    pub request_template: String,
    pub response_template: String
}

fn request_template() -> String {
    s!("\"{\"path\": $request.path, \"body\": $request.body}\"")
}

fn response_template() -> String {
    format!(r#"#set ($parsedPayload = $util.parseJson($input.json('$.output'))) $parsedPayload"#)
}

fn find_target_name(proxy: &Option<String>) -> String {
    match proxy {
        Some(f) => s!(f),
        None => s!("default")
    }
}


impl Route {

    pub fn new(spec: &RouteSpec) -> Route {

        let target_kind = match spec.proxy {
            Some(_) => TargetKind::Function,
            None => TargetKind::StepFunction
        };

        let target_name = find_target_name(&spec.proxy);
        let target_arn =  template::lambda_arn(&target_name);

        Route {
            method: spec.method.clone(),
            path: spec.path.clone(),
            gateway: spec.gateway.clone(),
            authorizer: spec.authorizer.clone(),
            target_kind: target_kind,
            target_arn: target_arn,
            stage: s!("test"),
            stage_variables: HashMap::new(),
            request_template: request_template(),
            response_template: response_template(),

        }
    }

}
