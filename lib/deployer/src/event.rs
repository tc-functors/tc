use compiler::Event;
use compiler::event::TargetKind;
use aws::eventbridge;
use aws::lambda;
use aws::Env;
use colored::Colorize;
use std::collections::HashMap;


async fn update_permissions(env: &Env, event: &Event) {

    for target in event.targets.clone() {

        match target.kind {
            TargetKind::Function => {
                let client = lambda::make_client(env).await;
                let principal = "events.amazonaws.com";
                let statement_id = &event.rule_name;
                let function_name = &target.name;
                let _ = lambda::add_permission_basic(client, function_name, principal, statement_id).await;
                println!("updating permission - function: {}", function_name);
            },
            _ => println!("Nothing to do!")
        }

    }
}

async fn create_event(env: &Env, event: &Event) {
    let Event {
        rule_name,
        bus,
        pattern,
        ..
    } = event;

    let client = eventbridge::make_client(&env).await;

    let pattern = serde_json::to_string(&pattern).unwrap();
    let _rule_arn = eventbridge::create_rule(&client, &bus, &rule_name, &pattern).await;

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

        let t = eventbridge::make_target(
            &target.id,
            &target.arn,
            &target.role_arn,
            &target.kind.to_str(),
            input_transformer,
            Some(appsync),
        );
        xs.push(t)
    }
    eventbridge::put_targets(&client, &bus, &rule_name, xs).await;
    update_permissions(env, &event).await;
}

pub async fn create(env: &Env, events: &HashMap<String, Event>) {
    for (_, event) in events {
        create_event(env, event).await;
    }
}

pub async fn delete_event(env: &Env, event: Event) {
    println!("Deleting event {}", &event.name.red());

    let client = eventbridge::make_client(&env).await;
    for target in event.targets {
        eventbridge::remove_target(&client, &event.bus, &event.rule_name, &target.id).await;
    }
    eventbridge::delete_rule(&client, &event.bus, &event.rule_name).await;
}

pub async fn delete(env: &Env, events: &HashMap<String, Event>) {
    for (_, event) in events {
        delete_event(env, event.clone()).await;
    }
}
