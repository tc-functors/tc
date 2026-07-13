use crate::Auth;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::{
    Client,
    Error,
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
            .behavior_version(BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(20))
            .build(),
    )
}

fn find_latest(xs: Vec<LayerVersionsListItem>, layer_name: &str) -> String {
    match xs.first() {
        Some(m) => match m.clone().layer_version_arn {
            Some(v) => v,
            _ => panic!("No layer version found"),
        },
        _ => {
            println!("{}: ", layer_name);
            std::panic::set_hook(Box::new(|_| {
                println!("Layer not found");
            }));
            panic!("Layer not found")
        }
    }
}

pub async fn find_version(client: Client, layer_name: &str) -> Result<String, Error> {
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

#[derive(Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub version: i64,
    pub created: String,
}

async fn list_by_token(client: &Client, token: &str) -> (Vec<Layer>, Option<String>) {
    let res = client.list_layers().marker(token).send().await.unwrap();
    let mut layers: Vec<Layer> = vec![];
    let xs = res.layers.unwrap().to_vec();
    for x in xs {
        let ver = x.latest_matching_version.unwrap();
        let layer = Layer {
            name: x.layer_name.unwrap(),
            version: ver.version,
            created: ver.created_date.unwrap(),
        };
        layers.push(layer);
    }
    (layers, res.next_marker)
}

pub async fn list(auth: &Auth) -> Vec<Layer> {
    let client = make_client(auth).await;
    let res = client.list_layers().send().await.unwrap();
    let mut layers: Vec<Layer> = vec![];
    let initial = res.layers.unwrap().to_vec();
    let mut token = res.next_marker;

    for x in initial {
        let ver = x.latest_matching_version.unwrap();
        let layer = Layer {
            name: x.layer_name.unwrap(),
            version: ver.version,
            created: ver.created_date.unwrap(),
        };
        layers.push(layer);
    }
    match token {
        Some(tk) => {
            token = Some(tk);
            while token.is_some() {
                let (xs, t) = list_by_token(&client, &token.unwrap()).await;
                layers.extend(xs.clone());
                token = t.clone();
                if let Some(x) = t {
                    if x.is_empty() {
                        break;
                    }
                }
            }
        }
        None => (),
    }
    layers
}
