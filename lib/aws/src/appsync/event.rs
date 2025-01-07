use aws_sdk_appsync::types::builders::EventBridgeDataSourceConfigBuilder;
use aws_sdk_appsync::types::{DataSourceType, EventBridgeDataSourceConfig};
use aws_sdk_appsync::Client;
use colored::Colorize;
use kit::*;

fn make_event_config(bus_arn: &str) -> EventBridgeDataSourceConfig {
    let v = EventBridgeDataSourceConfigBuilder::default();
    v.event_bus_arn(bus_arn).build().unwrap()
}

pub async fn update_datasource(
    client: &Client,
    api_id: &str,
    name: &str,
    bus_arn: &str,
    role_arn: &str,
) {
    println!("Updating datasource:event {}", name.blue());
    let event_config = make_event_config(bus_arn);
    let r = client
        .update_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::AmazonEventbridge)
        .event_bridge_config(event_config)
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
    println!("Creating datasource:event {}", name.green());
    let event_config = make_event_config(bus_arn);
    let r = client
        .create_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::AmazonEventbridge)
        .event_bridge_config(event_config)
        .service_role_arn(s!(role_arn))
        .send()
        .await;
    match r {
        Ok(_) => (),
        Err(_) => (),
    }
}
