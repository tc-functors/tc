use aws_sdk_cloudwatchlogs::{Client, Error};
use aws_sdk_cloudfront::types::DistributionConfig;
use aws_sdk_cloudfront::types::builders::DistributionConfigBuilder;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

pub async fn find_distribution() {

}

pub async fn create_distribution() {

}

pub async fn create_invalidation() {

}
