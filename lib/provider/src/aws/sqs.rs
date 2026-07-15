use super::constants;
use crate::Auth;
use aws_sdk_sqs::{
    Client,
    config,
    config::retry::{
        RetryConfig,
        RetryMode,
    },
    types::QueueAttributeName,
};
use std::collections::HashMap;

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

fn make_attributes() -> HashMap<QueueAttributeName, String> {
    let mut m: HashMap<QueueAttributeName, String> = HashMap::new();
    m.insert(QueueAttributeName::VisibilityTimeout, String::from("900"));
    m
}

async fn queue_exists(client: &Client, name: &str) -> bool {
    let r = client
        .get_queue_url()
        .queue_name(String::from(name))
        .send()
        .await;
    match r {
        Ok(res) => match res.queue_url {
            Some(_) => true,
            None => false,
        },
        Err(_) => false,
    }
}

pub async fn create_queue(client: &Client, name: &str) {
    let attrs = make_attributes();
    let exists = queue_exists(client, name).await;
    println!("Checking queue: exists {}", name);
    if !exists {
        let r = client
            .create_queue()
            .queue_name(String::from(name))
            .set_attributes(Some(attrs))
            .send()
            .await;
        match r {
            Ok(_) => (),
            Err(_) => panic!("{:?}", r),
        }
    }
}

pub async fn delete_queue(client: &Client, url: &str) {
    let _ = client
        .delete_queue()
        .queue_url(String::from(url))
        .send()
        .await;
}
