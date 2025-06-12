use authorizer::Auth;
use aws_sdk_sfn::{
    Client,
    config as sfn_config,
    config::retry::RetryConfig,
    operation::start_sync_execution::StartSyncExecutionOutput,
};
use std::panic;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        sfn_config::Builder::from(shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(7))
            .build(),
    )
}

pub async fn start_execution(client: Client, arn: &str, input: &str) -> String {
    println!("Invoking {}", arn);
    let res = client
        .start_execution()
        .state_machine_arn(arn.to_string())
        .input(input)
        .send()
        .await;
    match res {
        Ok(r) => r.execution_arn,
        Err(e) => {
            println!("{:?}", e);
            panic::set_hook(Box::new(|_| {
                println!("Error: Failed to invoke. Check payload or sandbox");
            }));
            panic!("Failed to invoke")
        }
    }
}

pub async fn _start_sync_execution(
    client: Client,
    arn: &str,
    input: &str,
    name: Option<String>,
) -> StartSyncExecutionOutput {
    let res = client
        .start_sync_execution()
        .state_machine_arn(arn.to_string())
        .input(input)
        .set_name(name)
        .send()
        .await;
    match res {
        Ok(r) => r,
        Err(e) => panic!("error: {:?}", e),
    }
}
