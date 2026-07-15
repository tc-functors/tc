use crate::Auth;
use aws_sdk_efs::{
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

pub async fn get_ap_arn(auth: &Auth, name: &str) -> Result<Option<String>, Error> {
    let client = make_client(auth).await;
    let res = client.describe_access_points().send().await;
    match res {
        Ok(r) => {
            match r.access_points {
                Some(xs) => {
                    for x in xs.iter() {
                        if &x.name.clone().unwrap() == name {
                            return Ok(x.clone().access_point_arn);
                        }
                    }
                }
                None => (),
            }
            return Ok(None);
        }
        Err(e) => panic!("{:?}", e),
    }
}
