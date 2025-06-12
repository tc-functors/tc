use authorizer::Auth;
use aws_sdk_lambda::{
    Client,
    Error,
    config as lambda_config,
    config::retry::RetryConfig,
};

use kit::*;
use std::{
    collections::HashMap,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        lambda_config::Builder::from(shared_config)
            .behavior_version(lambda_config::BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

pub async fn list_tags(client: Client, arn: &str) -> Result<HashMap<String, String>, Error> {
    let res = client.list_tags().resource(arn).send().await;

    match res {
        Ok(r) => {
            let maybe_tags = r.tags();
            match maybe_tags {
                Some(tags) => Ok(tags.clone()),
                None => Ok(HashMap::new()),
            }
        }
        Err(_) => Ok(HashMap::new()),
    }
}

pub struct Config {
    pub code_size: i64,
    pub timeout: i32,
    pub mem_size: i32,
    pub revision: String,
}

pub async fn find_config(client: &Client, name: &str) -> Option<Config> {
    let r = client
        .get_function_configuration()
        .function_name(s!(name))
        .send()
        .await;
    match r {
        Ok(res) => {
            let cfg = Config {
                code_size: res.code_size,
                timeout: res.timeout.unwrap(),
                mem_size: res.memory_size.unwrap(),
                revision: split_last(&res.revision_id.unwrap(), "-"),
            };
            Some(cfg)
        }
        Err(_e) => None,
    }
}

pub async fn find_function_layers(
    client: &Client,
    name: &str,
) -> Result<HashMap<String, i64>, Error> {
    let res = client
        .get_function_configuration()
        .function_name(name)
        .send()
        .await;

    let mut h: HashMap<String, i64> = HashMap::new();

    match res {
        Ok(r) => {
            match r.layers {
                Some(xs) => {
                    for x in xs {
                        h.insert(x.arn.unwrap(), x.code_size);
                    }
                }
                None => (),
            }
            Ok(h)
        }
        Err(_) => Ok(HashMap::new()),
    }
}
