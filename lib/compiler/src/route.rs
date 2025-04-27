use super::{
    spec::{
        RouteSpec,
        TopologySpec,
        Entity,
    },
    template,
};
use configurator::Config;
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
    pub target_kind: Entity,
    pub target_arn: String,
    pub stage: String,
    pub stage_variables: HashMap<String, String>,
    pub request_template: String,
    pub response_template: String,
}

fn request_template() -> String {
    s!("\"{\"path\": $request.path, \"body\": $request.body}\"")
}

fn response_template() -> String {
    format!(r#"#set ($parsedPayload = $util.parseJson($input.json('$.output'))) $parsedPayload"#)
}

fn find_target_arn(target_name: &str, target_kind: &Entity) -> String {
    match target_kind {
        Entity::Function => template::lambda_arn(&target_name),
        Entity::State => template::sfn_arn(&target_name),
        _ => template::lambda_arn(&target_name),
    }
}

impl Route {
    pub fn new(spec: &TopologySpec, rspec: &RouteSpec, _config: &Config) -> Route {
        let target_kind = match &rspec.proxy {
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
        let target_arn = find_target_arn(&target_name, &target_kind);

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
