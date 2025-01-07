use aws_sdk_appsync::types::builders::HttpDataSourceConfigBuilder;
use aws_sdk_appsync::types::{DataSourceType, HttpDataSourceConfig};
use aws_sdk_appsync::Client;
use colored::Colorize;
use kit::*;

fn make_http_config(url: &str) -> HttpDataSourceConfig {
    let v = HttpDataSourceConfigBuilder::default();
    v.endpoint(url).build()
}

pub async fn update_datasource(
    client: &Client,
    api_id: &str,
    name: &str,
    url: &str,
    role_arn: &str,
) {
    println!("Updating datasource:http {}", name.blue());
    let http_config = make_http_config(url);
    let r = client
        .update_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::Http)
        .http_config(http_config)
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
    url: &str,
    role_arn: &str,
) {
    println!("Creating datasource:http {}", name.green());
    let http_config = make_http_config(url);
    let r = client
        .create_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::Http)
        .http_config(http_config)
        .service_role_arn(s!(role_arn))
        .send()
        .await;
    match r {
        Ok(_) => (),
        Err(_) => (),
    }
}
