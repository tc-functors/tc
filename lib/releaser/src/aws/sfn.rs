use anyhow::Result;
use authorizer::Auth;
use aws_sdk_sfn::{
    Client,
    Error,
    config as sfn_config,
    config::retry::RetryConfig,
    types::{
        Tag,
        builders::TagBuilder,
    },
};
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        sfn_config::Builder::from(shared_config)
            .retry_config(RetryConfig::standard().with_max_attempts(7))
            .build(),
    )
}

fn make_tag(key: String, value: String) -> Tag {
    let tb = TagBuilder::default();
    tb.key(key).value(value).build()
}

fn make_tags(kvs: HashMap<String, String>) -> Vec<Tag> {
    let mut tags: Vec<Tag> = vec![];
    for (k, v) in kvs.into_iter() {
        let tag = make_tag(k, v);
        tags.push(tag);
    }
    tags
}

pub async fn update_tags(client: &Client, arn: &str, tags: HashMap<String, String>) -> Result<()> {
    let tags = make_tags(tags);
    client
        .tag_resource()
        .resource_arn(arn.to_string())
        .set_tags(Some(tags))
        .send()
        .await?;
    Ok(())
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

pub async fn get_tag(client: &Client, arn: &str, tag: String) -> String {
    let tags = list_tags(&client, arn).await.unwrap();
    match tags.get(&tag) {
        Some(v) => v.to_string(),
        None => "".to_string(),
    }
}
