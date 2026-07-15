use aws_sdk_dynamodb::{
    Client,
    config,
    config::retry::{RetryConfig, RetryMode},
    types::{
        AttributeDefinition,
        BillingMode,
        KeySchemaElement,
        KeyType,
        ScalarAttributeType
};

use crate::Auth;
use super::constants;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = auth.load().await;
    Client::from_conf(
        config::Builder::from(shared_config)
            .behavior_version(constants::behavior_version())
            .timeout_config(constants::timeout_config())
            .retry_config(
                RetryConfig::standard()
                    .with_retry_mode(RetryMode::Adaptive)
                    .with_max_attempts(constants::MAX_ATTEMPTS)
                    .with_initial_backoff(constants::INITIAL_BACKOFF)
                    .with_max_backoff(constants::MAX_BACKOFF)
            )
            .build(),
    )
}

fn make_hash_key_schema(hash_key: &str) -> KeySchemaElement {
     KeySchemaElement::builder()
        .attribute_name(hash_key)
        .key_type(KeyType::Hash)
        .build()
}

fn make_range_key_schema(range_key: &str) -> KeySchemaElement {
    KeySchemaElement::builder()
        .attribute_name(&range_key)
        .key_type(KeyType::Range)
        .build()
}

fn make_attr_definition(key: &str) -> AttributeDefinition {
    AttributeDefinition::builder()
        .attribute_name(String::from(key))
        .attribute_type(ScalarAttributeType::S)
        .build()
}


struct Schema {
    hash_key: Option<KeySchemaElement>,
    range_key: Option<KeySchemaElement>
}

pub async fn create_table(client: &Client, table_name: &str, schema: &Schema)  {
    println!("Creating table {}", table_name);
    let res = client
        .create_table()
        .table_name(table_name)
        .key_schema(hash_key_schema)
        .key_schema(range_key_schema)
        .attribute_definitions(hash_key_attribute)
        .attribute_definitions(range_key_attribute)
        .billing_mode(BillingMode::PayPerRequest)
        .send()
        .await;
}
