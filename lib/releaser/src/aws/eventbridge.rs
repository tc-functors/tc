use authorizer::Auth;
pub use aws_sdk_eventbridge::types::Target;
use aws_sdk_eventbridge::{
    Client,
    types::{
        AppSyncParameters, InputTransformer, RetryPolicy,
        builders::{RetryPolicyBuilder, TargetBuilder},
    },
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
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
