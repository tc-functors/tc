use colored::Colorize;
use compiler::Entity;
use composer::{
    Event,
};
use provider::{
    Auth,
    aws::{
        appsync,
        eventbridge,
        lambda,
    },
};
use std::collections::HashMap;

async fn update_permissions(auth: &Auth, event: &Event) {
    for target in event.targets.clone() {
        match target.entity {
            Entity::Function => {
                let client = lambda::make_client(auth).await;
                let principal = "events.amazonaws.com";
                let statement_id = &event.rule_name;
                let function_name = &target.name;
                let _ =
                    lambda::add_permission_basic(client, function_name, principal, statement_id)
                        .await;
                println!("Updating permission - function: {}", function_name.cyan());
            }
            _ => (),
        }
    }
}

fn should_prune() -> bool {
    match std::env::var("TC_PRUNE_EVENT_RULES") {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn create_target_dependencies(auth: &Auth, name: &str) -> String {
    let appsync_client = appsync::make_client(auth).await;
    let api_creds = appsync::events::find_api_creds(&appsync_client, name).await;
    if let Some(creds) = api_creds {
        println!("Creating Event Target dependencies {}", name);
        let client = eventbridge::make_client(auth).await;
        let endpoint = format!("{}/event", &creds.http_domain);
        eventbridge::find_or_create_api_destination(&client, name, &endpoint, &creds.api_key).await
    } else {
        println!("Skipping Event Target dependencies {}", name);
        String::from("invalid")
    }
}

async fn create_event(auth: &Auth, event: &Event, tags: &HashMap<String, String>) {
    let Event {
        rule_name,
        bus,
        name,
        pattern,
        ..
    } = event;

    let client = eventbridge::make_client(&auth).await;

    let pattern = serde_json::to_string(&pattern).unwrap();
    let _rule_arn = eventbridge::create_rule(&client, &bus, &rule_name, &pattern, tags).await;

    println!(
        "Creating event: {} ({})",
        &name.green(),
        &event.targets.len()
    );

    if should_prune() {
        let existing_targets = eventbridge::list_targets(&client, &bus, &rule_name).await;
        for id in existing_targets {
            println!("Removing event target {}", &id);
            eventbridge::remove_targets(&client, &bus, &rule_name, &id).await;
        }
    }

    let mut xs: Vec<eventbridge::Target> = vec![];
    for target in &event.targets {
        let appsync = eventbridge::make_appsync_params(&target.name);

        let input_transformer = match target.input_template.clone() {
            Some(_) => Some(eventbridge::make_input_transformer(
                target.input_paths_map.clone(),
                target.input_template.clone(),
            )),
            None => None,
        };

        let target_arn = if &target.entity.to_str() == "channel" {
            create_target_dependencies(auth, &target.name).await
        } else {
            String::from(&target.arn)
        };

        if target_arn.is_empty() || target_arn == "none" {
            println!(
                "WARN: Event Target {}'s arn is invalid: {}. perhaps retry ?",
                &target.id, &target_arn
            );
            std::process::exit(1);
        }

        let t = eventbridge::make_target(
            &target.id,
            &target_arn,
            &target.role_arn,
            &target.entity.to_str(),
            input_transformer,
            Some(appsync),
        );
        xs.push(t)
    }
    eventbridge::put_targets(&client, &bus, &rule_name, xs).await;
    update_permissions(auth, &event).await;
}

pub async fn create(auth: &Auth, events: &HashMap<String, Event>, tags: &HashMap<String, String>) {
    for (_, event) in events {
        if !&event.skip {
            create_event(auth, event, tags).await;
        }
    }
}

pub async fn delete_event(auth: &Auth, event: Event) {
    println!("Deleting event {}", &event.rule_name);

    let client = eventbridge::make_client(&auth).await;
    for target in event.targets {
        eventbridge::remove_target(&client, &event.bus, &event.rule_name, &target.id).await;
    }
    eventbridge::delete_rule(&client, &event.bus, &event.rule_name).await;
}

pub async fn delete(auth: &Auth, events: &HashMap<String, Event>) {
    for (_, event) in events {
        if !&event.skip {
            delete_event(auth, event.clone()).await;
        }
    }
}

pub async fn update(
    auth: &Auth,
    events: &HashMap<String, Event>,
    tags: &HashMap<String, String>,
    c: &str,
) {
    if let Some(event) = events.get(c) {
        create_event(auth, event, tags).await;
    }
}
