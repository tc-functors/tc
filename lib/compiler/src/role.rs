use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};
use crate::template;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RoleKind {
    StepFunction,
    Function,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub path: String,
    pub trust: String,
    pub arn: String,
    pub policy_name: String,
    pub policy: String,
    pub policy_arn: String,
}

fn read_policy(path: &str) -> String {
    u::slurp(path)
}

fn function_trust_policy() -> String {
    format!(
        r#"{{"Version": "2012-10-17",
    "Statement": [
        {{
            "Effect": "Allow",
            "Principal": {{
                "Service": [
                    "lambda.amazonaws.com",
                    "events.amazonaws.com",
                    "logs.amazonaws.com",
                    "scheduler.amazonaws.com",
                    "appsync.amazonaws.com",
                    "apigateway.amazonaws.com"
                ]
            }},
            "Action": "sts:AssumeRole"
        }}
    ]
     }}"#
    )
}

fn sfn_trust_policy() -> String {
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


fn find_default_trust_policy(kind: RoleKind) -> String {
    match kind {
        RoleKind::Function => function_trust_policy(),
        RoleKind::StepFunction => sfn_trust_policy()
    }
}

fn find_default_role(kind: RoleKind) -> String {
    match kind {
        RoleKind::Function => s!("tc-base-lambda-role"),
        RoleKind::StepFunction => s!("tc-base-sfn-role")
    }
}

pub fn default(kind: RoleKind) -> Role {
    let role_name = find_default_role(kind);
    Role {
        name: role_name.clone(),
        path: s!("provided"),
        trust: s!("provided"),
        arn: template::role_arn(&role_name),
        policy: s!("provided"),
        policy_name: s!("tc-base-lambda-policy"),
        policy_arn: template::policy_arn("tc-base-lambda-policy")
    }
}

impl Role {

    pub fn new(kind: RoleKind, role_file: &str, role_name: &str, policy_name: &str) -> Role {

        if u::file_exists(&role_file) {
            Role {
                name: s!(role_name),
                path: s!(role_file),
                trust: find_default_trust_policy(kind),
                arn: template::role_arn(&role_name),
                policy: read_policy(&role_file),
                policy_name: policy_name.to_string(),
                policy_arn: template::policy_arn(&policy_name)
            }
        } else {
            panic!("Cannot find role_file");
        }
    }


}
