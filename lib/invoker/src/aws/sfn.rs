use authorizer::Auth;
use aws_sdk_sfn::{
    Client,
    config as sfn_config,
    config::retry::RetryConfig,
    Error,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        sfn_config::Builder::from(shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(7))
            .build(),
    )
}

pub async fn start_execution(
    client: Client, 
    arn: &str, 
    input: &str
) -> Result<(String, Option<String>), Error> {
    println!("Invoking Standard State Machine with ARN: {}", arn);
    let response = client
        .start_execution()
        .state_machine_arn(arn.to_string())
        .input(input)
        .send()
        .await?;
    
    let execution_arn = response.execution_arn;
    let start_date_ts = Some(response.start_date);
    let start_date: Option<String> = start_date_ts.map(|dt| dt.to_string());

    Ok((execution_arn, start_date))
}

pub async fn start_sync_execution(
    client: Client,
    arn: &str,
    input: &str,
    name: Option<String>,
) -> Result<(String, Option<String>), Error> {
    println!("Invoking Express State Machine with ARN: {}", arn);
    let response = client
        .start_sync_execution()
        .state_machine_arn(arn.to_string())
        .input(input)
        .set_name(name)
        .send()
        .await?;

    let execution_arn = response.execution_arn;
    let output = response.output;

    Ok((execution_arn, output))
}
