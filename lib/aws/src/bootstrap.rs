use super::Env;
use crate::iam;
use crate::iam::Role;

async fn make_role(env: &Env, name: &str) -> Role {
    let policy_doc = match name {
        "lambda" => env.base_lambda_policy(),
        "sfn" => env.base_sfn_policy(),
        "event" => env.base_event_policy(),
        "api" => env.base_api_policy(),
        "appsync" => env.base_appsync_policy(),
        _ => panic!("No such policy"),
    };
    let client = iam::make_client(env).await;
    let role_fqn = env.base_role(name);
    let policy_fqn = env.base_policy(name);
    let policy_arn = env.policy_arn(&policy_fqn);
    Role {
        client: client.clone(),
        name: role_fqn,
        trust_policy: env.base_trust_policy(),
        policy_arn: policy_arn,
        policy_name: policy_fqn,
        policy_doc: policy_doc,
    }
}

pub async fn show_role(env: &Env, name: &str) {
    let role = make_role(env, name).await;
    let j = kit::json_value(&role.policy_doc);
    println!("{}", kit::pretty_json(&j));
}

pub async fn create_role(env: &Env, name: &str) {
    let role = make_role(env, name).await;
    let out = role.create().await;
    println!("{:?}", out);
}

pub async fn delete_role(env: &Env, name: &str) {
    let role = make_role(env, name).await;
    let out = role.delete().await;
    println!("{:?}", out);
}
