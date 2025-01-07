use crate::resolver::{Flow, Logs};
use aws::cloudwatch;
use aws::iam;
use aws::iam::Role;
use aws::sfn;
use aws::sfn::StateMachine;
use aws::Env;
use std::collections::HashMap;

pub async fn create(env: &Env, flow: Flow) {
    let name = &flow.name;
    let definition = serde_json::to_string(&flow.definition).unwrap();
    let mode = sfn::make_mode(&flow.mode);

    if !definition.is_empty() {
        let client = sfn::make_client(env).await;
        let iam_client = iam::make_client(env).await;
        let role = flow.role.clone();
        let role_arn = match role {
            Some(ref r) => String::from(&env.role_arn(&r.name)),
            None => String::from(&flow.default_role),
        };
        if let Some(ro) = role {
            println!("Creating sfn-role {}", ro.name);
            let r = Role {
                client: iam_client,
                name: ro.name,
                trust_policy: ro.trust.to_string(),
                policy_arn: ro.policy_arn,
                policy_name: ro.policy_name,
                policy_doc: ro.policy.to_string(),
            };
            let _ = r.create().await;
        };

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: role_arn,
            tags: flow.clone().tags,
        };

        let arn = &flow.arn;
        sf.create_or_update(arn).await;
    }
}

pub async fn delete(env: &Env, flow: Flow) {
    let name = flow.clone().name;

    let definition = serde_json::to_string(&flow.definition).unwrap();
    let mode = sfn::make_mode(&flow.mode);

    if !definition.is_empty() {
        let client = sfn::make_client(env).await;

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: flow.clone().default_role,
            tags: flow.clone().tags,
        };

        sf.delete(&flow.arn).await.unwrap();
    }
}

pub async fn update_tags(env: &Env, name: &str, tags: HashMap<String, String>) {
    let client = sfn::make_client(env).await;
    let sfn_arn = env.sfn_arn(name);
    let _ = sfn::update_tags(&client, &sfn_arn, tags).await;
}

pub async fn enable_logs(env: &Env, sfn_arn: &str, logs: Logs) {
    let sfn_client = sfn::make_client(env).await;
    let cw_client = cloudwatch::make_client(env).await;

    let aggregator = logs.aggregator;

    cloudwatch::create_log_group(cw_client.clone(), &aggregator.states)
        .await
        .unwrap();
    let _ = sfn::enable_logging(sfn_client, sfn_arn, &aggregator.arn).await;
}

pub async fn disable_logs(env: &Env, sfn_arn: &str) {
    let sfn_client = sfn::make_client(env).await;
    sfn::disable_logging(sfn_client, sfn_arn).await.unwrap();
}
