use authorizer::Auth;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::{
    Client,
    config as lambda_config,
    config::retry::RetryConfig,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        lambda_config::Builder::from(shared_config)
            .behavior_version(BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

pub async fn delete(client: &Client, arn: &str) {
    println!("Deleting {}", arn);
    let _ = client
        .delete_function()
        .function_name(arn)
        .send()
        .await
        .unwrap();
}
