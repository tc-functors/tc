use anyhow::Result;
use authorizer::Auth;
use aws_sdk_lambda::{
    Client,
    config as lambda_config,
    config::retry::RetryConfig,
    types::LayerVersionsListItem,
};
use kit::*;
use std::panic;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        lambda_config::Builder::from(shared_config)
            .behavior_version(lambda_config::BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

pub async fn get_code_url(client: &Client, arn: &str) -> Option<String> {
    let r = client.get_layer_version_by_arn().arn(s!(arn)).send().await;
    match r {
        Ok(res) => {
            let content = res.content.unwrap();
            content.location
        }
        Err(e) => panic!("{}", e),
    }
}

fn find_latest(xs: Vec<LayerVersionsListItem>, layer_name: &str) -> String {
    match xs.first() {
        Some(m) => match m.clone().layer_version_arn {
            Some(v) => v,
            _ => panic!("No layer version found"),
        },
        _ => {
            println!("{}: ", layer_name);
            panic::set_hook(Box::new(|_| {
                println!("Layer not found");
            }));
            panic!("Layer not found")
        }
    }
}

pub async fn find_version(client: Client, layer_name: &str) -> Result<String> {
    let res = client
        .list_layer_versions()
        .layer_name(layer_name)
        .send()
        .await?;

    match res.layer_versions {
        Some(xs) => Ok(find_latest(xs, layer_name)),
        None => panic!("No layer found"),
    }
}
