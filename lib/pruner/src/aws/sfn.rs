use authorizer::Auth;
use aws_sdk_sfn::{
    Client,
    config as sfn_config,
    config::retry::RetryConfig,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        sfn_config::Builder::from(shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(7))
            .build(),
    )
}

pub async fn delete(client: &Client, arn: &str) {
    println!("Deleting {}", arn);
    let _ = client
        .delete_state_machine()
        .state_machine_arn(arn.to_string())
        .send()
        .await
        .unwrap();
}
