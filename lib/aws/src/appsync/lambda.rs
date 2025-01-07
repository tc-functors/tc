use aws_sdk_appsync::types::builders::LambdaDataSourceConfigBuilder;
use aws_sdk_appsync::types::{DataSourceType, LambdaDataSourceConfig};
use aws_sdk_appsync::Client;
use colored::Colorize;
use kit::*;

fn make_lambda_config(lambda_arn: &str) -> LambdaDataSourceConfig {
    let v = LambdaDataSourceConfigBuilder::default();
    v.lambda_function_arn(lambda_arn).build().unwrap()
}

pub async fn update_datasource(
    client: &Client,
    api_id: &str,
    name: &str,
    lambda_arn: &str,
    role_arn: &str,
) {
    println!("Updating datasource:function {}", name.blue());
    let lambda_config = make_lambda_config(lambda_arn);
    let r = client
        .update_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::AwsLambda)
        .lambda_config(lambda_config)
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
    lambda_arn: &str,
    role_arn: &str,
) {
    println!("Creating datasource:function {}", name.green());
    let lambda_config = make_lambda_config(lambda_arn);
    let r = client
        .create_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .r#type(DataSourceType::AwsLambda)
        .lambda_config(lambda_config)
        .service_role_arn(s!(role_arn))
        .send()
        .await;
    match r {
        Ok(_) => (),
        Err(_) => (),
    }
}
