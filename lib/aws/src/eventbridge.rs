use aws_sdk_eventbridge::types::builders::AppSyncParametersBuilder;
use aws_sdk_eventbridge::types::builders::InputTransformerBuilder;
use aws_sdk_eventbridge::types::builders::PutEventsRequestEntryBuilder;
use aws_sdk_eventbridge::types::builders::TargetBuilder;
use aws_sdk_eventbridge::types::AppSyncParameters;
use aws_sdk_eventbridge::types::InputTransformer;
use aws_sdk_eventbridge::types::PutEventsRequestEntry;
use aws_sdk_eventbridge::types::{Rule, RuleState, Target};
use aws_sdk_eventbridge::Client;
use std::collections::HashMap;

use super::Env;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

pub fn make_input_transformer(
    input_paths_map: Option<HashMap<String, String>>,
    input_template: Option<String>,
) -> InputTransformer {
    let it = InputTransformerBuilder::default();
    it.set_input_paths_map(input_paths_map)
        .set_input_template(input_template)
        .build()
        .unwrap()
}

pub fn make_appsync_params(op: &str) -> AppSyncParameters {
    let params = AppSyncParametersBuilder::default();
    params.graph_ql_operation(op).build()
}

pub fn make_target(
    id: &str,
    arn: &str,
    role_arn: &str,
    kind: &str,
    input_transformer: Option<InputTransformer>,
    appsync: Option<AppSyncParameters>,
) -> Target {
    let target = TargetBuilder::default();

    match kind {
        "sfn" => target.id(id).arn(arn).role_arn(role_arn).build().unwrap(),
        "lambda" => target.id(id).arn(arn).build().unwrap(),
        "appsync" => target
            .id(id)
            .arn(String::from(arn))
            .role_arn(role_arn)
            .set_input_transformer(input_transformer)
            .set_app_sync_parameters(appsync)
            .build()
            .unwrap(),
        _ => target.id(id).arn(arn).role_arn(role_arn).build().unwrap(),
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub client: Client,
    pub name: String,
    pub rule_name: String,
    pub bus: String,
    pub pattern: String,
    pub role: String,
    pub target: Target,
}

impl Event {
    pub async fn _list_rules(self) -> Vec<Rule> {
        let s = self.clone();
        let r = self
            .client
            .list_rules()
            .event_bus_name(s.bus)
            .send()
            .await
            .unwrap();
        match r.rules {
            Some(p) => p,
            None => panic!("No rules"),
        }
    }

    pub async fn create_rule(self) -> String {
        let s = self.clone();
        let r = s
            .client
            .put_rule()
            .event_bus_name(self.bus)
            .name(self.rule_name)
            .event_pattern(self.pattern)
            .state(RuleState::Enabled)
            .send()
            .await
            .unwrap();
        match r.rule_arn {
            Some(p) => p,
            None => panic!("oops"),
        }
    }

    pub async fn put_target(self) {
        self.client
            .put_targets()
            .event_bus_name(self.bus)
            .rule(self.rule_name)
            .targets(self.target)
            .send()
            .await
            .unwrap();
    }

    pub async fn delete_rule(self) {
        self.clone()
            .client
            .delete_rule()
            .event_bus_name(self.bus)
            .name(self.rule_name)
            .force(true)
            .send()
            .await
            .unwrap();
    }

    pub async fn remove_targets(self, id: &str) {
        let s = self.clone();
        s.client
            .remove_targets()
            .event_bus_name(self.clone().bus)
            .rule(self.clone().rule_name)
            .ids(id.to_string())
            .force(true)
            .send()
            .await
            .unwrap();
    }
}

fn make_event(bus: &str, detail_type: &str, source: &str, detail: &str) -> PutEventsRequestEntry {
    let e = PutEventsRequestEntryBuilder::default();
    let event = e
        .source(source)
        .detail_type(detail_type)
        .detail(detail)
        .event_bus_name(bus)
        .build();
    event
}

pub async fn put_event(client: Client, bus: &str, detail_type: &str, source: &str, detail: &str) {
    let event = make_event(bus, detail_type, source, detail);
    let resp = client.put_events().entries(event).send().await;
    println!("{:?}", resp);
}

pub async fn put_target(client: Client, bus: String, rule_name: String, target: Target) {
    client
        .put_targets()
        .event_bus_name(bus)
        .rule(rule_name)
        .targets(target)
        .send()
        .await
        .unwrap();
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
