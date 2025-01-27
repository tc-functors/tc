use kit::*;
use kit as u;
use serde_derive::{Deserialize, Serialize};
use crate::template;


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

// TODO: Implement ABAC
pub fn read_policy(path: &str) -> String {
    u::slurp(path)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub path: String,
    pub trust: String,
    pub arn: String,
    pub policy_name: String,
    pub policy: String,
    pub policy_arn: String
}

impl Role {

    pub fn new(role_file: Option<String>, namespace: &str, function_name: &str) -> Role {
        let abbr = if function_name.chars().count() > 15 {
            u::abbreviate(function_name, "-")
        } else {
            function_name.to_string()
        };
        let policy_name = format!("tc-{}-{{{{sandbox}}}}-{}-policy", namespace, abbr);
        let role_name = format!("tc-{}-{{{{sandbox}}}}-{}-role", namespace, abbr);

        match role_file {
            Some(f) => Role {
                name: role_name.clone(),
                path: f.clone(),
                trust: default_trust_policy(),
                arn: template::role_arn(&role_name),
                policy: read_policy(&f),
                policy_name: policy_name.clone(),
                policy_arn: template::policy_arn(&policy_name)
            },
            None => Role {
                name: s!("tc-base-lambda-role"),
                path: s!("provided"),
                trust: s!("provided"),
                arn: template::role_arn("tc-base-lambda-role"),
                policy: s!("provided"),
                policy_name: s!("tc-base-lambda-policy"),
                policy_arn: template::policy_arn("tc-base-lambda-policy")
            }
        }
    }
}
