use kit as u;
use kit::*;

use super::template;
use crate::spec::function::Provider;
use crate::topology::role::Role;

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

fn find_default_role(provider: Provider) -> String {
    match provider {
        Provider::Lambda => s!("tc-base-lambda-role"),
        Provider::Fargate => s!("tc-base-task-role"),
    }
}

pub fn default(provider: Option<Provider>) -> Role {
    let provider = match provider {
        Some(p) => p,
        None => Provider::Lambda,
    };
    let role_name = find_default_role(provider);
    Role {
        name: role_name.clone(),
        path: s!("provided"),
        trust: s!("provided"),
        arn: template::role_arn(&role_name),
        policy: s!("provided"),
        policy_name: s!("tc-base-lambda-policy"),
        policy_arn: template::policy_arn("tc-base-lambda-policy"),
    }
}

pub fn use_given(role_name: &str) -> Role {
    Role {
        name: role_name.to_string(),
        path: s!("provided"),
        trust: s!("provided"),
        arn: template::role_arn(&role_name),
        policy: s!("provided"),
        policy_name: s!("tc-base-lambda-policy"),
        policy_arn: template::policy_arn("tc-base-lambda-policy"),
    }
}

pub fn make(role_file: &str, role_name: &str, policy_name: &str) -> Role {
    if u::file_exists(&role_file) {
        Role {
            name: s!(role_name),
            path: s!(role_file),
            trust: function_trust_policy(),
            arn: template::role_arn(&role_name),
            policy: read_policy(&role_file),
            policy_name: policy_name.to_string(),
            policy_arn: template::policy_arn(&policy_name),
        }
    } else {
        panic!("Cannot find role_file");
    }
}
