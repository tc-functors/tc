use crate::Auth;
pub use aws_sdk_eventbridge::types::{
    RuleState,
    Target,
};
use aws_sdk_eventbridge::{
    Client,
    types::{
        ApiDestinationHttpMethod,
        AppSyncParameters,
        ConnectionAuthorizationType,
        CreateConnectionAuthRequestParameters,
        InputTransformer,
        RetryPolicy,
        PutEventsRequestEntry,
        Tag,
        builders::{
            AppSyncParametersBuilder,
            CreateConnectionApiKeyAuthRequestParametersBuilder,
            CreateConnectionAuthRequestParametersBuilder,
            PutEventsRequestEntryBuilder,
            InputTransformerBuilder,
            RetryPolicyBuilder,
            TagBuilder,
            TargetBuilder,
        },
    },
};
use kit::*;
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

fn make_tag(key: String, value: String) -> Tag {
    let tb = TagBuilder::default();
    tb.key(key).value(value).build().unwrap()
}

fn make_tags(kvs: HashMap<String, String>) -> Vec<Tag> {
    let mut tags: Vec<Tag> = vec![];
    for (k, v) in kvs.into_iter() {
        let tag = make_tag(k, v);
        tags.push(tag);
    }
    tags
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

pub fn make_retry_policy() -> RetryPolicy {
    let ret = RetryPolicyBuilder::default();
    ret.maximum_retry_attempts(1).build()
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
    let retry_policy = make_retry_policy();

    match kind {
        "sfn" | "stepfunction" => target
            .id(id)
            .arn(arn)
            .role_arn(role_arn)
            .retry_policy(retry_policy)
            .build()
            .unwrap(),
        "lambda" | "function" => target
            .id(id)
            .arn(arn)
            .retry_policy(retry_policy)
            .build()
            .unwrap(),
        "appsync" | "mutation" => target
            .id(id)
            .arn(String::from(arn))
            .role_arn(role_arn)
            .set_input_transformer(input_transformer)
            .set_app_sync_parameters(appsync)
            .retry_policy(retry_policy)
            .build()
            .unwrap(),
        "channel" => target
            .id(id)
            .arn(String::from(arn))
            .role_arn(role_arn)
            .set_input_transformer(input_transformer)
            .retry_policy(retry_policy)
            .build()
            .unwrap(),
        _ => target
            .id(id)
            .arn(arn)
            .role_arn(role_arn)
            .retry_policy(retry_policy)
            .build()
            .unwrap(),
    }
}

pub async fn create_rule(
    client: &Client,
    bus: &str,
    rule_name: &str,
    pattern: &str,
    tags: &HashMap<String, String>,
) -> String {
    let tags = make_tags(tags.clone());
    let r = client
        .put_rule()
        .event_bus_name(s!(bus))
        .name(s!(rule_name))
        .event_pattern(s!(pattern))
        .state(RuleState::Enabled)
        .set_tags(Some(tags))
        .send()
        .await
        .unwrap();
    match r.rule_arn {
        Some(p) => p,
        None => panic!("oops"),
    }
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

pub async fn put_targets(client: &Client, bus: &str, rule_name: &str, targets: Vec<Target>) {
    client
        .put_targets()
        .event_bus_name(s!(bus))
        .rule(s!(rule_name))
        .set_targets(Some(targets))
        .send()
        .await
        .unwrap();
}

pub async fn remove_target(client: &Client, bus: &str, rule_name: &str, id: &str) {
    client
        .remove_targets()
        .event_bus_name(s!(bus))
        .rule(s!(rule_name))
        .ids(s!(id))
        .force(true)
        .send()
        .await
        .unwrap();
}

pub async fn delete_rule(client: &Client, bus: &str, rule_name: &str) {
    client
        .delete_rule()
        .event_bus_name(s!(bus))
        .name(s!(rule_name))
        .force(true)
        .send()
        .await
        .unwrap();
}

pub async fn list_targets(client: &Client, bus: &str, rule_name: &str) -> Vec<String> {
    let res = client
        .list_targets_by_rule()
        .event_bus_name(bus)
        .rule(rule_name)
        .send()
        .await
        .unwrap();
    let maybe_targets = res.targets;
    match maybe_targets {
        Some(v) => {
            let mut xs: Vec<String> = vec![];
            for x in v {
                xs.push(x.id().to_string())
            }
            xs
        }
        None => vec![],
    }
}

pub async fn remove_targets(client: &Client, bus: &str, rule_name: &str, target_id: &str) {
    client
        .remove_targets()
        .event_bus_name(bus.to_string())
        .rule(rule_name.to_string())
        .ids(target_id.to_string())
        .force(true)
        .send()
        .await
        .unwrap();
}

// api destination

fn make_auth_params(api_key: &str) -> CreateConnectionAuthRequestParameters {
    let ret = CreateConnectionApiKeyAuthRequestParametersBuilder::default();
    let api_key_auth_params = ret
        .api_key_name(s!("x-api-key"))
        .api_key_value(s!(api_key))
        .build()
        .unwrap();
    let b = CreateConnectionAuthRequestParametersBuilder::default();
    b.api_key_auth_parameters(api_key_auth_params).build()
}

async fn create_connection(client: &Client, name: &str, api_key: &str) -> String {
    let auth_params = make_auth_params(api_key);
    let res = client
        .create_connection()
        .name(s!(name))
        .authorization_type(ConnectionAuthorizationType::ApiKey)
        .auth_parameters(auth_params)
        .send()
        .await;
    res.unwrap().connection_arn.unwrap()
}

async fn find_connection(client: &Client, name: &str) -> Option<String> {
    let res = client.describe_connection().name(s!(name)).send().await;
    match res {
        Ok(r) => r.connection_arn,
        Err(_) => None,
    }
}

async fn find_or_create_connection(client: &Client, name: &str, api_key: &str) -> String {
    match find_connection(client, name).await {
        Some(c) => c,
        None => create_connection(client, name, api_key).await,
    }
}

async fn find_api_destination(client: &Client, name: &str) -> Option<String> {
    let res = client
        .describe_api_destination()
        .name(s!(name))
        .send()
        .await;
    match res {
        Ok(r) => r.api_destination_arn,
        Err(_) => None,
    }
}

async fn create_api_destination(
    client: &Client,
    name: &str,
    connection_arn: &str,
    endpoint: &str,
) -> String {
    let res = client
        .create_api_destination()
        .name(s!(name))
        .connection_arn(s!(connection_arn))
        .invocation_endpoint(s!(endpoint))
        .http_method(ApiDestinationHttpMethod::Post)
        .send()
        .await;
    res.unwrap().api_destination_arn.unwrap()
}

pub async fn find_or_create_api_destination(
    client: &Client,
    name: &str,
    endpoint: &str,
    api_key: &str,
) -> String {
    match find_api_destination(client, name).await {
        Some(api_dest_arn) => api_dest_arn,
        None => {
            println!("Creating API destination {}", name);
            let connection_arn = find_or_create_connection(client, name, api_key).await;
            create_api_destination(client, name, &connection_arn, endpoint).await
        }
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

pub async fn put_event(
    client: Client,
    bus: &str,
    detail_type: &str,
    source: &str,
    detail: &str,
) -> String {
    let event = make_event(bus, detail_type, source, detail);
    let resp = client.put_events().entries(event).send().await;
    resp.unwrap()
        .entries
        .expect("Failed")
        .first()
        .unwrap()
        .event_id
        .clone()
        .unwrap()
}
