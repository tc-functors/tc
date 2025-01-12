use super::{Context, Topology};
use aws::Env;
use kit as u;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
                    "logs.amazonaws.com"
                ]
            }},
            "Action": "sts:AssumeRole"
        }}
    ]
     }}"#
    )
}

fn render_policy(s: &str, env: Env) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    let account = &env.account();
    let region = &env.region();
    table.insert("env", &env.name);
    table.insert("account", account);
    table.insert("region", region);
    u::stencil(s, table)
}

pub fn read_policy(env: Env, path: &str) -> Value {
    let data = u::slurp(path);
    u::json_value(&render_policy(&data, env))
}

fn read_trust_policy(env: Env) -> Value {
    let data = default_trust_policy();
    u::json_value(&render_policy(&data, env))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub trust: Value,
    pub policy: Value,
    pub policy_name: String,
    pub policy_arn: String,
}

impl Role {
    pub fn new(env: &Env, role_file: &str, role_name: &str, policy_name: &str) -> Role {
        Role {
            name: role_name.to_owned(),
            trust: read_trust_policy(env.clone()),
            policy: read_policy(env.clone(), role_file),
            policy_name: policy_name.to_string(),
            policy_arn: env.policy_arn(&policy_name),
        }
    }
}

pub fn make(context: &Context, topology: &Topology) -> HashMap<String, Role> {
    let Context { env, .. } = context;
    let mut roles: HashMap<String, Role> = HashMap::new();
    for (_, f) in &topology.functions {
        let role_file = &f.role.path;
        if u::file_exists(role_file) {
            let role_name = context.render(&f.role.name);
            let policy_name = context.render(&f.role.policy_name);
            let role = Role::new(env, role_file, &role_name, &policy_name);
            roles.insert(f.clone().name, role);
        }
    }
    roles
}
