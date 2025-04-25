use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use configurator::Config;
use kit::*;
use super::template;
use super::spec::{TopologySpec, RouteSpec};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TargetKind {
    Function,
    StepFunction
}

impl TargetKind {

    pub fn to_str(&self) -> String {
        match self {
            TargetKind::Function => s!("function"),
            TargetKind::StepFunction => s!("sfn")
        }
    }
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


fn find_target_arn(target_name: &str, target_kind: &TargetKind) -> String {
    match target_kind {
        TargetKind::Function => template::lambda_arn(&target_name),
        TargetKind::StepFunction => template::sfn_arn(&target_name)
    }
}

impl Route {

    pub fn new(spec: &TopologySpec, rspec: &RouteSpec, _config: &Config) -> Route {

        let target_kind = match &rspec.proxy {
            Some(_) => TargetKind::Function,
            None => match rspec.function {
                Some(_) => TargetKind::Function,
                None => TargetKind::StepFunction
            }
        };

        let target_name = match &rspec.proxy {
            Some(f) => s!(f),
            None => match &rspec.function {
                Some(x) => s!(x),
                None => template::topology_fqn(&spec.name, spec.hyphenated_names)
            }
        };
        let target_arn =  find_target_arn(&target_name, &target_kind);

        Route {
            method: rspec.method.clone(),
            path: rspec.path.clone(),
            gateway: rspec.gateway.clone(),
            authorizer: rspec.authorizer.clone(),
            target_kind: target_kind,
            target_arn: target_arn,
            stage: s!("test"),
            stage_variables: HashMap::new(),
            request_template: request_template(),
            response_template: response_template(),
        }
    }

}
