use aws_sdk_cloudfront::{Client, Error};
use aws_sdk_cloudfront::types::{DistributionConfig, Aliases};
use aws_sdk_cloudfront::types::builders::DistributionConfigBuilder;
use aws_sdk_cloudfront::types::builders::AliasesBuilder;
use aws_sdk_cloudfront::types::FunctionRuntime;
use aws_sdk_cloudfront::types::{Origin, S3OriginConfig, Origins};
use aws_sdk_cloudfront::types::builders::{S3OriginConfigBuilder, OriginBuilder};
use aws_sdk_cloudfront::types::CloudFrontOriginAccessIdentityConfig;
use aws_sdk_cloudfront::types::builders::CloudFrontOriginAccessIdentityConfigBuilder;
use aws_sdk_cloudfront::types::builders::OriginsBuilder;
use std::collections::HashMap;
use kit::*;


pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

fn make_aliases() -> Aliases {
    let it = DistributionConfigBuilder::default();
}

fn make_origin(id: &str, bucket: &str, oai_id: &str) -> Origin {
    let bucket_path = format!("{}.s3.amazonaws.com");
    let oai_path = format!("origin-access-identity/cloudfront/{}", oai_id);
    let s3b = S3OriginConfigBuilder::default;

    let s3_origin_config = s3b
        .origin_access_identity(oai_path)
        .build()

    let it = OriginBuilder::default();
    it
        .id(id)
        .domain_name(bucket_path)
        .s3_origin_config(s3_origin_config)
        .build()
}

fn make_origins(domain: &str, paths: Vec<String>, oai_id: &str) -> Origins {
    let it = OriginsBuilder::default();
    let items = paths.map(|m| make_origin()
    it
        .quantity(paths.len())
        .items()
}

pub fn make_dist_config(
    caller_ref: &str,
    origin_domain: &str,
    origin_paths: Vec<String>,
    oai_id: &str

) -> DistributionConfig {
    let it = DistributionConfigBuilder::default();
    let origins = make_origins()
    let aliases = make_aliases();
    it
        .caller_reference(reference)
        .aliases(aliases)
        .origins(origins)
        .build()
}


async fn list_distributions(client: &Client) -> HashMap<String, String> {
    let res = client
        .list_distributions()
        .send()
        .await
        .unwrap();
    let items = res.distribution_list.items.unwrap();

    let mut h: HashMap<String, String> = HashMap::new();
    for item in items {
        h.insert(item.domain_name, item.id);
    }
    h
}

async fn find_distribution(client: &Client, domain: &str) -> Option<String> {
    let dists = list_distributions(client).await;
    dists.get(domain)
}

async fn update_distribution(
    client: &client,
    id: &str,
    dc: DistributionConfig
) {
    client
        .update_distribution()
        .distribution_config(dc)
        .send()
        .await;
}

async fn create_distribution(client: &Client, dc: DistributionConfig) {
    let res = client
        .create_distribution()
        .distribution_config(dc)
        .send()
        .await;
    println!("{}", res);
}

async fn create_or_update_distribution(
    client: &Client,
    domains: Vec<String>,
    dc: DistributionConfig
) {
    let maybe_dist = find_distribution(client, domain).await;
    match maybe_dist {
        Some(d) => update_distribution(client, &d, dc).await,
        None => create_distribution(client, dc).await
    }
}

pub async fn create_invalidation(client: &Client) {

}


// origin access identity

fn make_oai_config(
    caller_ref: &str,
    comment: &str
) -> CloudFrontOriginAccessIdentityConfig {

    let it = CloudFrontOriginAccessIdentityConfigBuilder::default();
    it.caller_reference(caller_ref).comment(comment).build()
}

pub async fn create_oai(client: &Client, caller_ref: &str) -> String {
    let cfg = make_oai_config(caller_ref, "");

    let res = client
        .create_cloud_front_origin_access_identity()
        .cloud_front_origin_access_identity_config(cfg)
        .send()
        .await
        .unwrap();
    res.cloud_front_origin_access_identity_config.id
}


// origin access control
pub async fn create_oac() {

}



// Edge function

fn make_function_config(name: &str) -> FunctionConfig {
    let it = FunctionConfigBuilder::default();
    it
        .comment(name)
        .runtime(FunctionRuntime::CloudfrontJs20)
        .build()
}


async fn find_function(client: &Client, name: &str) {

}


async fn update_function() {

}


async fn create_function(client: &Client, name: &str) {
    let fc = make_function_config(name);
    let res = client
        .create_function()
        .function_config(fc)
}

async fn create_or_update_function() {

}


pub async fn test_function(client: &Client) {
    let res = client
        .test_function()
        .name(name)
        .if_match()
}
