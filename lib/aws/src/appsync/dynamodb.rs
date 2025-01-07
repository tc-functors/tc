use aws_sdk_appsync::types::builders::DynamodbDataSourceConfigBuilder;
use aws_sdk_appsync::types::{DataSourceType, DynamodbDataSourceConfig};
use aws_sdk_appsync::Client;
use colored::Colorize;
use kit::*;

fn make_config(table_name: &str) -> DynamodbDataSourceConfig {
    let v = DynamodbDataSourceConfigBuilder::default();
    v.table_name(table_name)
        .aws_region(s!("us-west-2"))
        .build()
        .unwrap()
}

pub async fn update_datasource(
    client: &Client,
    api_id: &str,
    name: &str,
    table_name: &str,
    role_arn: &str,
) {
    println!("Updating datasource:table {}", name.blue());
    let config = make_config(table_name);
    let r = client
        .update_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::AmazonDynamodb)
        .dynamodb_config(config)
        .service_role_arn(s!(role_arn))
        .send()
        .await;
    match r {
        Ok(_) => (),
        Err(_) => (),
    }
}

pub async fn create_datasource(
    client: &Client,
    api_id: &str,
    name: &str,
    bus_arn: &str,
    role_arn: &str,
) {
    println!("Creating datasource:table {}", name.green());
    let config = make_config(bus_arn);
    let r = client
        .create_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::AmazonDynamodb)
        .dynamodb_config(config)
        .service_role_arn(s!(role_arn))
        .send()
        .await;
    match r {
        Ok(_) => (),
        Err(_) => (),
    }
}
