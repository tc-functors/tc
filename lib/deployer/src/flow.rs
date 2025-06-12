use crate::aws::{cloudwatch, iam, iam::Role, sfn, sfn::StateMachine};
use authorizer::Auth;
use compiler::{Flow, LogConfig};
use std::collections::HashMap;

pub async fn update_definition(auth: &Auth, tags: &HashMap<String, String>, flow: Flow) {
    let name = &flow.name;
    let definition = serde_json::to_string(&flow.definition).unwrap();
    let mode = sfn::make_mode(&flow.mode);

    if !definition.is_empty() {
        let client = sfn::make_client(auth).await;
        let role = flow.role.clone();
        let role_arn = role.arn;

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: role_arn,
            tags: tags.clone(),
        };

        let arn = &flow.arn;
        sf.create_or_update(arn).await;
    }
}

pub async fn create(auth: &Auth, tags: &HashMap<String, String>, flow: Flow) {
    let name = &flow.name;
    let definition = serde_json::to_string(&flow.definition).unwrap();
    let mode = sfn::make_mode(&flow.mode);

    if !definition.is_empty() {
        let client = sfn::make_client(auth).await;
        let iam_client = iam::make_client(auth).await;
        let role = flow.role.clone();
        let role_arn = role.arn;

        let r = Role {
            client: iam_client,
            name: role.name,
            trust_policy: role.trust.to_string(),
            policy_arn: role.policy_arn,
            policy_name: role.policy_name,
            policy_doc: role.policy.to_string(),
        };
        let _ = r.create_or_update().await;

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: role_arn,
            tags: tags.clone(),
        };

        let arn = &flow.arn;
        sf.create_or_update(arn).await;
    }
}

pub async fn delete(auth: &Auth, flow: Flow) {
    let name = flow.clone().name;

    let definition = serde_json::to_string(&flow.definition).unwrap();
    let mode = sfn::make_mode(&flow.mode);

    if !definition.is_empty() {
        let client = sfn::make_client(auth).await;

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: flow.role.arn.to_string(),
            tags: HashMap::new(),
        };

        sf.delete(&flow.arn).await.unwrap();
    }
}

pub async fn update_tags(auth: &Auth, name: &str, tags: HashMap<String, String>) {
    let client = sfn::make_client(auth).await;
    let sfn_arn = auth.sfn_arn(name);
    let _ = sfn::update_tags(&client, &sfn_arn, tags).await;
}

pub async fn enable_logs(auth: &Auth, sfn_arn: &str, logs: LogConfig, flow: &Flow) {
    let sfn_client = sfn::make_client(auth).await;
    let cw_client = cloudwatch::make_client(auth).await;

    let aggregator = logs.aggregator;

    let include_exec_data = match std::env::var("TC_SFN_DEBUG") {
        Ok(_) => true,
        Err(_) => {
            if flow.mode == "Express" {
                true
            } else {
                false
            }
        }
    };

    cloudwatch::create_log_group(cw_client.clone(), &aggregator.states)
        .await
        .unwrap();
    tracing::debug!(
        "Updating log-config {} ({}) include_exec_data: {}",
        flow.name,
        flow.mode,
        include_exec_data
    );
    let _ = sfn::enable_logging(sfn_client, sfn_arn, &aggregator.arn, include_exec_data).await;
}

pub async fn disable_logs(auth: &Auth, sfn_arn: &str) {
    let sfn_client = sfn::make_client(auth).await;
    sfn::disable_logging(sfn_client, sfn_arn).await.unwrap();
}
