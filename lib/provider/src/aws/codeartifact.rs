use super::constants;
use crate::Auth;
use aws_sdk_codeartifact::{
    Client,
    config,
    config::retry::{
        RetryConfig,
        RetryMode,
    },
};
use kit::*;

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
                    .with_max_backoff(constants::MAX_BACKOFF),
            )
            .build(),
    )
}

pub async fn get_auth_token(client: &Client, domain: &str, owner: &str) -> String {
    let res = client
        .get_authorization_token()
        .domain(s!(domain))
        .domain_owner(s!(owner))
        .send()
        .await;
    res.unwrap().authorization_token.unwrap()
}
