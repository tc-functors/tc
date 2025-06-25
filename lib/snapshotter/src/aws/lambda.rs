use authorizer::Auth;
use aws_sdk_lambda::{
    Client,
    Error,
    config as lambda_config,
    config::retry::RetryConfig,
};
use std::collections::HashMap;

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
