use crate::Auth;
use aws_sdk_ssm::{
    Client,
    config,
    config::retry::{RetryConfig, RetryMode},
    Error,
};
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

pub async fn get(client: Client, key: &str) -> Result<String, Error> {
    let r = client
        .get_parameter()
        .name(key)
        .with_decryption(true)
        .send()
        .await;

    let res = match r {
        Ok(v) => v.parameter.unwrap().value.unwrap(),
        Err(_) => String::from(""),
    };

    Ok(res)
}
