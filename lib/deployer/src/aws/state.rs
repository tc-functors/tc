use composer::Flow;
use kit as u;
use kit::*;
use provider::{
    Auth,
    aws::{
        cloudwatch,
        sfn,
        sfn::StateMachine,
    },
};
use std::collections::HashMap;

pub async fn update_definition(auth: &Auth, tags: &HashMap<String, String>, flow: &Flow) {
    let Flow {
        name, arn, role_arn, ..
    } = flow;
    let definition = serde_json::to_string(&flow.definition).unwrap();
    let mode = sfn::make_mode(&flow.mode);

    if !definition.is_empty() {
        let client = sfn::make_client(auth).await;

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: role_arn.clone(),
            tags: tags.clone(),
        };

        sf.create_or_update(arn).await;
    }
}

pub async fn create(auth: &Auth, flow: &Flow, tags: &HashMap<String, String>) {
    let name = &flow.name;
    let definition = serde_json::to_string(&flow.definition).unwrap();
    let mode = sfn::make_mode(&flow.mode);

    if !definition.is_empty() {
        let client = sfn::make_client(auth).await;
        let role_arn = &flow.role_arn;

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: role_arn.to_string(),
            tags: tags.clone(),
        };

        let arn = &flow.arn;
        sf.create_or_update(arn).await;

        update_logs(auth, arn, flow).await;
    }
}

pub async fn delete(auth: &Auth, flow: &Flow) {
    let Flow {
        name,
        definition,
        mode,
        arn,
        role_arn,
        ..
    } = flow;

    let definition = serde_json::to_string(definition).unwrap();
    let mode = sfn::make_mode(mode);

    disable_logs(&auth, arn).await;

    if !definition.is_empty() {
        let client = sfn::make_client(auth).await;

        let sf = StateMachine {
            name: name.clone(),
            client: client,
            mode: mode,
            definition: definition,
            role_arn: role_arn.to_string(),
            tags: HashMap::new(),
        };

        sf.delete(arn).await.unwrap();
    }
}

pub async fn update_tags(auth: &Auth, name: &str, tags: &HashMap<String, String>) {
    let client = sfn::make_client(auth).await;
    let sfn_arn = auth.sfn_arn(name);
    let _ = sfn::update_tags(&client, &sfn_arn, tags.clone()).await;
}

pub async fn update_logs(auth: &Auth, sfn_arn: &str, flow: &Flow) {
    let sfn_client = sfn::make_client(auth).await;
    let cw_client = cloudwatch::make_client(auth).await;
    let Flow {
        name,
        mode,
        log_config,
        ..
    } = flow;

    let include_exec_data = match std::env::var("TC_SFN_DEBUG") {
        Ok(_) => true,
        Err(_) => {
            if mode == "Express" {
                true
            } else {
                false
            }
        }
    };

    cloudwatch::create_log_group(cw_client.clone(), &log_config.group)
        .await
        .unwrap();
    println!(
        "Updating state {} (logging) mode: {} tracing: {}",
        name, mode, include_exec_data
    );
    let _ = sfn::enable_logging(
        sfn_client,
        sfn_arn,
        &log_config.group_arn,
        include_exec_data,
    )
    .await;
}

pub async fn disable_logs(auth: &Auth, sfn_arn: &str) {
    let sfn_client = sfn::make_client(auth).await;
    sfn::disable_logging(sfn_client, sfn_arn).await.unwrap();
}

pub async fn update(auth: &Auth, flow: &Flow, tags: &HashMap<String, String>, component: &str) {
    match component {
        "definition" => update_definition(auth, tags, flow).await,
        "tags" => update_tags(auth, &flow.name, tags).await,
        "logs" => update_logs(&auth, &flow.arn, flow).await,
        _ => (),
    }
}

pub async fn freeze(auth: &Auth, fqn: &str) {
    let client = sfn::make_client(auth).await;
    let arn = auth.sfn_arn(fqn);
    let version = sfn::get_tag(&client, &arn, s!("version")).await;
    if &version != "0.0.1" && !&version.is_empty() {
        println!("Unfreezing {} ({})", fqn, version);
        let kv = u::kv("freeze", "true");
        let _ = sfn::update_tags(&client, &arn, kv).await;
    }
}

pub async fn unfreeze(auth: &Auth, fqn: &str) {
    let client = sfn::make_client(auth).await;
    let arn = auth.sfn_arn(fqn);
    let version = sfn::get_tag(&client, &arn, s!("version")).await;
    if &version != "0.0.1" && !&version.is_empty() {
        println!("Unfreezing {} ({})", fqn, version);
        let kv = u::kv("freeze", "true");
        let _ = sfn::update_tags(&client, &arn, kv).await;
    }
}

pub async fn create_dry_run(flow: &Flow) {
    println!("Creating state: {}", flow.name);
}
