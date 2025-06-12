use authorizer::Auth;
use aws_sdk_sfn::{Client, Error, config as sfn_config, config::retry::RetryConfig};
use std::collections::HashMap;

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
