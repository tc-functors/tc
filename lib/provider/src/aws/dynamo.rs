use crate::Env;
use aws_sdk_dynamodb::{
    Client,
    types::AttributeValue,
};

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

pub fn attr(s: &str) -> AttributeValue {
    AttributeValue::S(String::from(s))
}
