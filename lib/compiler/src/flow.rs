use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use super::spec::TopologySpec;
use super::version;

fn default_trust_policy() -> String {
    format!(
        r#"{{"Version": "2012-10-17",
    "Statement": [
        {{
            "Effect": "Allow",
            "Principal": {{
                "Service": [
                    "lambda.amazonaws.com",
                    "events.amazonaws.com",
                    "states.amazonaws.com",
                    "logs.amazonaws.com",
                    "apigateway.amazonaws.com",
                    "appsync.amazonaws.com",
                    "scheduler.amazonaws.com"
                ]
            }},
            "Action": "sts:AssumeRole"
        }}
    ]
     }}"#
    )
}

pub fn read_policy(path: &str) -> String {
    u::slurp(path)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub path: String,
    pub trust: String,
    pub policy_name: String,
    pub policy: String,
    pub policy_arn: Option<String>,
}


impl Role {

    pub fn new(infra_dir: &str, namespace: &str) -> Role {
        let role_file = format!("{}/roles/sfn.json", infra_dir);

        if u::file_exists(&role_file) {
            Role {
                name: format!("tc-{}-sfn-role", namespace),
                path: role_file.clone(),
                trust: default_trust_policy(),
                policy: read_policy(&role_file),
                policy_name: format!("tc-{}-sfn-policy", namespace),
                policy_arn: None
            }
        } else {
            Role {
                name: s!("tc-base-lambda-role"),
                path: s!("provided"),
                trust: s!("provided"),
                policy: s!("provided"),
                policy_name: s!("tc-base-lambda-policy"),
                policy_arn: None
            }

        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flow {
    pub name: String,
    pub tags: HashMap<String, String>,
    pub definition: Value,
    pub mode: String,
    pub role: Role
}

fn read_definition(dir: &str, def: Value) -> Value {
    match def.as_str() {
        Some(p) => {
            let path = format!("{}/{}", dir, &p);
            if (path.ends_with(".json") || path.ends_with(".yml")) && u::file_exists(&path) {
                let data = u::slurp(&path);
                u::json_value(&data)
            } else {
                def
            }
        }
        None => def,
    }
}

fn make_tags(namespace: &str) -> HashMap<String, String> {
    let tc_version = option_env!("PROJECT_VERSION")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();
    let version = version::current_semver(namespace);
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(s!("namespace"), s!(namespace));
    h.insert(s!("sandbox"), format!("{{{{sandbox}}}}"));
    h.insert(s!("version"), version);
    h.insert(s!("git_branch"), version::branch_name());
    h.insert(s!("deployer"), s!("tc"));
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("tc_version"), tc_version);
    h
}

impl Flow {

    pub fn new(dir: &str, infra_dir: &str, namespace: &str, spec: &TopologySpec) -> Option<Flow> {

        let def = match &spec.flow {
            Some(f) => Some(read_definition(dir, f.clone())),
            None => spec.states.to_owned(),
        };

        let mode = match &spec.mode {
            Some(m) => m.to_string(),
            None => s!("Express")
        };

        let role = Role::new(infra_dir, namespace);

        let tags = make_tags(namespace);

        match def {
            Some(definition) => Some(
                Flow {
                    name: s!(namespace),
                    tags: tags,
                    definition: definition,
                    mode: mode,
                    role: role
                }),
            None => None
        }
    }
}
