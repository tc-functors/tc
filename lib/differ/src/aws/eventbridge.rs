use authorizer::Auth;
pub use aws_sdk_eventbridge::types::{
    Rule,
};
use aws_sdk_eventbridge::{
    Client,
};


pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn list_rules(client: Client, bus: String, prefix: String) -> Vec<Rule> {
    let r = client
        .list_rules()
        .event_bus_name(bus)
        .name_prefix(prefix)
        .send()
        .await
        .unwrap();
    r.rules.unwrap()
}

pub async fn get_target(client: Client, bus: String, rule: String) -> String {
    let r = client
        .list_targets_by_rule()
        .event_bus_name(bus)
        .rule(rule)
        .send()
        .await
        .unwrap();

    match r.targets {
        Some(targets) => targets.first().unwrap().arn.clone(),
        None => String::from(""),
    }
}
