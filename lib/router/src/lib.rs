use provider::Auth;
use provider::aws::eventbridge;
use configurator::Config;

fn target_id(name: &str) -> String {
    format!("{}_target", name)
}

pub fn role_arn(acc: &str, namespace: &str, sandbox: &str, id: &str) -> String {
    format!(
        "arn:aws:iam::{}:role/{}-{}-{}-role",
        acc, namespace, sandbox, id
    )
}

pub async fn route(auth: &Auth, event_id: &str, service: &str, sandbox: &str, rule: &str) {
    let client = eventbridge::make_client(auth).await;
    let config = Config::new(None);
    let bus = &config.aws.eventbridge.bus;
    let target_name = format!("{}_{}", service, sandbox);
    let target_id = target_id(event_id);
    let target_arn = auth.sfn_arn(&target_name);
    let role = role_arn(&auth.account, service, sandbox, "event");
    let target = eventbridge::make_target(&target_id, event_id, &target_arn, &role, None, None);
    println!("Routing {} to {}", event_id, target_name);
    eventbridge::put_target(client, bus.to_string(), rule.to_string(), target).await;
}
