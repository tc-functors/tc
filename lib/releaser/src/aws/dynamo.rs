use authorizer::Auth;
use aws_sdk_dynamodb::{
    Client,
    types::AttributeValue,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub fn attr(s: &str) -> AttributeValue {
    AttributeValue::S(String::from(s))
}
