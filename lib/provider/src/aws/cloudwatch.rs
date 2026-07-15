use crate::Auth;
use aws_sdk_cloudwatchlogs::{
    Client,
    config,
    config::retry::{RetryConfig, RetryMode},
    Error,
};
use kit::*;
use super::constants;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        config::Builder::from(shared_config)
            .behavior_version(constants::behavior_version())
            .timeout_config(constants::timeout_config())
            .retry_config(
                RetryConfig::standard()
                    .with_retry_mode(RetryMode::Adaptive)
                    .with_max_attempts(constants::MAX_ATTEMPTS)
                    .with_initial_backoff(constants::INITIAL_BACKOFF)
                    .with_max_backoff(constants::MAX_BACKOFF)
            )
            .build(),
    )
}

pub async fn create_log_group(client: Client, group: &str) -> Result<(), Error> {
    let r = client
        .create_log_group()
        .log_group_name(s!(group))
        .send()
        .await;

    match r {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub async fn _create_subscription_filter(
    client: Client,
    group: &str,
    filter_name: &str,
    filter: &str,
    lambda_arn: &str,
) -> Result<(), Error> {
    let r = client
        .put_subscription_filter()
        .log_group_name(s!(group))
        .filter_name(s!(filter_name))
        .filter_pattern(s!(filter))
        .destination_arn(s!(lambda_arn))
        .send()
        .await;

    match r {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub async fn _delete_subscription_filter(
    client: Client,
    group: &str,
    filter_name: &str,
) -> Result<(), Error> {
    let r = client
        .delete_subscription_filter()
        .log_group_name(s!(group))
        .filter_name(s!(filter_name))
        .send()
        .await;

    match r {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}
