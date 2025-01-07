use aws_sdk_cloudwatchlogs::{Client, Error};

use super::Env;
use kit::*;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
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
