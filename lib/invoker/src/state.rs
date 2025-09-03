use crate::aws::sfn;
use authorizer::Auth;
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use std::collections::HashMap;

fn get_id(arn: &str) -> &str {
    let xs = arn.split(":").collect::<Vec<&str>>();
    let last = xs.last();
    match last {
        Some(x) => x,
        _ => "",
    }
}

fn name_of(arn: &str) -> String {
    let parts: Vec<&str> = arn.split(":").collect();
    u::nth(parts, 6)
}

pub fn open_execution(auth: &Auth, mode: &str, exec_arn: &str) {
    let name = &name_of(exec_arn);
    let id = get_id(exec_arn);
    let url = if mode == "Express" {
        println!("Invoking Express StateMachine {}", name);
        auth.sfn_url_express(&exec_arn)
    } else {
        println!("Invoking Standard StateMachine {}", name);
        auth.sfn_url(name, id)
    };
    println!("{}", url);
    open::that(url).unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Response {
    error: Option<String>,
    cause: Option<String>,
    output: Value,
}

pub async fn execute_state_machine(auth: &Auth, name: &str, payload: &str, mode: &str, dumb: bool) {
    let client = sfn::make_client(auth).await;
    let arn = auth.sfn_arn(name);
    let exec_arn = sfn::start_execution(client.clone(), &arn, &payload).await;
    if !dumb {
        open_execution(auth, mode, &exec_arn);
    }
}

pub async fn execute_sync_state_machine(auth: &Auth, name: &str, payload: &str) -> Option<String> {
    let client = sfn::make_client(auth).await;
    let arn = auth.sfn_arn(name);
    let result = sfn::start_sync_execution(client.clone(), &arn, &payload, Some(name.to_string())).await;
    result
}

fn build_vars(auth: &Auth) -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = HashMap::new();
    vars.insert(s!("account"), auth.account.clone());
    vars.insert(s!("region"), auth.region.clone());
    vars
}

pub fn augment_payload(payload_str: &str, vars: HashMap<String, String>) -> String {
    u::merge_json(payload_str, &vars)
}

pub async fn invoke(auth: &Auth, name: &str, payload: &str, mode: &str, dumb: bool) {
    let vars = build_vars(auth);
    let payload = augment_payload(payload, vars);

    if mode == "Express" {
        let result = execute_sync_state_machine(auth, name, &payload).await;
        println!("{:?}", result);
    } else {
        execute_state_machine(auth, name, &payload, mode, dumb).await;
    };
}

pub async fn invoke_emulator(auth: &Auth, dir: &str, definition: &str, fqn: &str, payload: &str) {
    let Auth { region, .. } = auth;
    let role = "arn:aws:iam::012345678901:role/DummyRole";
    let arn = auth.sfn_arn(fqn);
    let cmd_pre = format!(
        "AWS_REGION={} AWS_PROFILE={} aws stepfunctions --endpoint http://localhost:8083",
        region, &auth.name
    );

    let def_cmd = format!(
        r#"{cmd_pre} create-state-machine --definition '{}' --name {} --role-arn {} "#,
        definition, fqn, role
    );
    println!("{}", &def_cmd);
    u::runcmd_stream(&def_cmd, &dir);

    let payload = serde_json::to_string(payload).unwrap();
    let exec_name = u::uuid_str();
    let start_cmd = format!(
        r#"{cmd_pre} start-execution --state-machine-arn {} --name {} --input '{}'"#,
        &arn, &exec_name, &payload
    );
    u::sh(&start_cmd, &dir);

    let exec_arn = auth.sfn_exec_arn(fqn, &exec_name);

    let desc_cmd = format!("{cmd_pre} describe-execution --execution-arn {}", &exec_arn);
    let desc = u::sh(&desc_cmd, &dir);
    println!("{}", desc);
}
