use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::{Context, Topology};
use crate::resolver::role;
use crate::resolver::role::Role;
use crate::resolver::vars;
use aws::Env;

fn render_template(env: &Env, tags: HashMap<String, String>, s: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    let account = &env.account();
    let region = &env.region();
    table.insert("account", account);
    table.insert("region", region);
    table.insert("env", &env.name);
    table.insert("namespace", &tags.get("namespace").unwrap());
    table.insert("sandbox", &tags.get("sandbox").unwrap());
    u::stencil(s, table)
}

fn trust_policy() -> Value {
    let a = format!(
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
    );

    u::json_value(&a)
}

fn make_role(env: &Env, path: &str, name: &str) -> Role {
    let role_name = format!("tc-{}-sfn-role", name);
    let policy_name = format!("tc-{}-sfn-policy", name);
    Role {
        name: s!(&role_name),
        trust: trust_policy(),
        policy: role::read_policy(env.clone(), path),
        policy_name: policy_name.clone(),
        policy_arn: env.policy_arn(&policy_name),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flow {
    pub name: String,
    pub arn: String,
    pub tags: HashMap<String, String>,
    pub definition: Value,
    pub mode: String,
    pub role: Option<Role>,
    pub default_role: String,
}

impl Flow {
    pub fn new(
        env: &Env,
        name: &str,
        mode: &str,
        definition: &Value,
        tags: HashMap<String, String>,
        role: Option<Role>,
        default_role: &str,
    ) -> Flow {
        let states_str = definition.to_string();
        let data = render_template(env, tags.clone(), &states_str);
        let definition = u::json_value(&data);

        Flow {
            name: s!(name),
            arn: env.sfn_arn(name),
            tags: tags,
            definition: definition,
            mode: s!(mode),
            role: role,
            default_role: s!(default_role),
        }
    }
}

pub fn make(
    context: &Context,
    topology: &Topology,
    mode: &str,
    tags: &HashMap<String, String>,
) -> Option<Flow> {
    let _vars = vars::default_vars(&context, "", "", "");

    let Context { env, name, .. } = context;

    let role = match &topology.role {
        Some(p) => Some(make_role(&env, &p, &name)),
        None => None,
    };

    let role_name = env.base_role("sfn");
    let default_role = &env.role_arn(&role_name);
    match &topology.flow {
        Some(def) => {
            let flow = Flow::new(&env, &name, mode, def, tags.clone(), role, default_role);
            Some(flow)
        }
        None => None,
    }
}
