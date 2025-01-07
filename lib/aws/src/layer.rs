use super::Env;
use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::config as lambda_config;
use aws_sdk_lambda::config::retry::RetryConfig;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::types::builders::LayerVersionContentInputBuilder;
use aws_sdk_lambda::types::{LayerVersionContentInput, LayerVersionsListItem, Runtime};
use aws_sdk_lambda::Client;
use kit::*;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::panic;
use tabled::Tabled;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::from_conf(
        lambda_config::Builder::from(&shared_config)
            .behavior_version(BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

pub fn make_blob_from_str(payload: &str) -> Blob {
    let buffer = payload.as_bytes();
    Blob::new(buffer)
}

fn make_blob(payload_file: &str) -> Blob {
    if file_exists(payload_file) {
        let f = File::open(payload_file).unwrap();
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        Blob::new(buffer)
    } else {
        make_blob_from_str("test")
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

pub async fn find_latest_version(client: &Client, layer_name: &str) -> i64 {
    let res = client
        .list_layer_versions()
        .layer_name(layer_name)
        .send()
        .await;
    match res {
        Ok(r) => match r.layer_versions {
            Some(xs) => {
                if xs.len() > 0 {
                    xs.first().unwrap().version
                } else {
                    0
                }
            }
            _ => 0,
        },
        Err(_) => 0,
    }
}

fn make_code(code: Blob) -> LayerVersionContentInput {
    let f = LayerVersionContentInputBuilder::default();
    f.zip_file(code).build()
}

pub fn make_runtimes(lang: &str) -> Vec<Runtime> {
    match lang {
        "java11" => vec![Runtime::Java11],
        "ruby2.7" => vec![Runtime::Ruby27],
        "python3.7" => vec![Runtime::Python37],
        "python3.8" => vec![Runtime::Python38],
        "python3.9" => vec![Runtime::Python39],
        "python3.10" => vec![Runtime::Python310, Runtime::Python311, Runtime::Python312],
        "python3.11" => vec![Runtime::Python310, Runtime::Python311, Runtime::Python312],
        "python3.12" => vec![Runtime::Python310, Runtime::Python311, Runtime::Python312],
        "provided" => vec![Runtime::Provided],
        "providedal2" => vec![Runtime::Providedal2],
        "go" => vec!["provided.al2023".into()],
        "janet" => vec!["provided.al2023".into()],
        "rust" => vec!["provided.al2023".into()],
        "ruby3.2" => vec!["ruby3.2".into()],
        _ => vec![Runtime::Provided],
    }
}

pub async fn publish(client: &Client, layer_name: &str, zipfile: &str, lang: &str) -> i64 {
    let blob = make_blob(zipfile);
    let content = make_code(blob);
    let runtimes = make_runtimes(lang);
    let r = client
        .publish_layer_version()
        .layer_name(layer_name)
        .content(content)
        .set_compatible_runtimes(Some(runtimes))
        .send()
        .await
        .unwrap();
    r.version
}

#[derive(Tabled, Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub version: i64,
    pub created_date: String,
}

pub async fn add_permission(client: &Client, layer_name: &str, version: i64) {
    let _ = client
        .add_layer_version_permission()
        .layer_name(s!(layer_name))
        .version_number(version)
        .statement_id(uuid_str())
        .action(s!("lambda:GetLayerVersion"))
        .principal(s!("*"))
        .send()
        .await
        .unwrap();
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

pub async fn list(client: Client, filter: Vec<String>) -> Vec<Layer> {
    let res = client
        .list_layers()
        .max_items(50)
        .send()
        .await
        .unwrap()
        .layers;

    let mut layers: Vec<Layer> = vec![];
    match res {
        Some(xs) => {
            for x in xs.to_vec() {
                let ver = x.latest_matching_version.unwrap();
                let layer_name = x.layer_name.unwrap();
                if filter.contains(&layer_name) {
                    let layer = Layer {
                        name: layer_name,
                        version: ver.version,
                        created_date: ver.created_date.unwrap(),
                    };
                    layers.push(layer);
                }
            }
        }
        None => (),
    }
    layers
}
