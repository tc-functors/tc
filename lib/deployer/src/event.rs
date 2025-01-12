use resolver::Event;
use aws::eventbridge;
use aws::lambda;
use aws::Env;
use colored::Colorize;

async fn make_event(env: &Env, event: Event) -> eventbridge::Event {
    let Event {
        name,
        rule_name,
        bus,
        target,
        pattern,
        ..
    } = event;

    let client = eventbridge::make_client(&env).await;
    let appsync = eventbridge::make_appsync_params(&target.name);
    let input_transformer = match target.input_template.clone() {
        Some(_) => Some(eventbridge::make_input_transformer(
            target.input_paths_map,
            target.input_template,
        )),
        None => None,
    };
    let aws_target = eventbridge::make_target(
        &target.id,
        &target.arn,
        &target.role_arn,
        &target.kind,
        input_transformer,
        Some(appsync),
    );
    eventbridge::Event {
        client: client,
        name: name,
        rule_name: rule_name,
        bus: bus,
        role: String::from(&target.role_arn),
        target: aws_target,
        pattern: serde_json::to_string(&pattern).unwrap(),
    }
}

async fn update_permissions(env: &Env, event: &Event) {
    let kind = &event.target.kind;
    match kind.as_ref() {
        "function" | "lambda" => {
            let client = lambda::make_client(env).await;
            let principal = "events.amazonaws.com";
            let statement_id = &event.rule_name;
            let function_name = &event.target.name;
            let _ = lambda::add_permission_basic(client, function_name, principal, statement_id).await;
            println!("updating permission - function: {}", function_name);
        },
        _ => println!("Nothing to do!")
    }
}

pub async fn create_event(env: &Env, event: Event) {
    println!("Creating event {}", &event.name.green());
    let target_event = make_event(env, event.clone()).await;

    let target_arn = &event.target.arn;
    if !target_arn.is_empty() {
        let rule_arn = target_event.clone().create_rule().await;

        if !rule_arn.is_empty() {
            target_event.clone().put_target().await;
        }
        update_permissions(env, &event).await;
    }
}

pub async fn create(env: &Env, events: Vec<Event>) {
    for event in events {
        create_event(env, event).await;
    }
}

pub async fn delete_event(env: &Env, event: Event) {
    println!("Deleting event {}", &event.name.red());

    let target_event = make_event(env, event.clone()).await;
    target_event.clone().remove_targets(&event.target.id).await;
    target_event.clone().delete_rule().await;
}

pub async fn delete(env: &Env, events: Vec<Event>) {
    for event in events {
        delete_event(env, event).await;
    }
}
