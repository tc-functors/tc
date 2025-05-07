use authorizer::Auth;
use aws_sdk_sfn::{
    Client,
    Error,
    config as sfn_config,
    config::retry::RetryConfig,
};
use kit::*;
use std::{
    collections::HashMap,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        sfn_config::Builder::from(shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(7))
            .build(),
    )
}

pub async fn list_tags(client: &Client, arn: &str) -> Result<HashMap<String, String>, Error> {
    let res = client
        .list_tags_for_resource()
        .resource_arn(arn.to_string())
        .send()
        .await;

    match res {
        Ok(r) => match r.tags {
            Some(xs) => {
                let mut h: HashMap<String, String> = HashMap::new();
                for tag in xs {
                    let k = tag.key().unwrap().to_string();
                    let v = tag.value().unwrap().to_string();
                    h.insert(k, v);
                }
                Ok(h)
            }
            _ => Ok(HashMap::new()),
        },

        Err(_) => Ok(HashMap::new()),
    }
}

pub async fn list(client: Client) -> Vec<HashMap<String, String>> {
    let res = client
        .clone()
        .list_state_machines()
        .max_results(1000)
        .send()
        .await
        .unwrap();
    let sfns = res.state_machines;
    let mut out: Vec<HashMap<String, String>> = vec![];
    for sfn in sfns {
        let mut h: HashMap<String, String> = HashMap::new();
        let arn = sfn.state_machine_arn;
        h.insert(s!("type"), sfn.r#type.as_str().to_string());
        let tags = list_tags(&client, &arn).await.unwrap();
        let namespace = tags.get("namespace");
        match namespace {
            Some(name) => {
                if !name.is_empty() {
                    h.insert(s!("version"), safe_unwrap(tags.get("version")));
                    h.insert(s!("namespace"), name.to_string());
                    h.insert(s!("sandbox"), safe_unwrap(tags.get("sandbox")));
                    h.insert(s!("updated_at"), safe_unwrap(tags.get("updated_at")));
                    out.push(h);
                }
            }
            None => (),
        }
    }
    out
}
