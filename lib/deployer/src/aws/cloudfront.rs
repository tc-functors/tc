use aws_sdk_cloudfront::{Client};
use aws_sdk_cloudfront::types::{DistributionConfig, Aliases};
use aws_sdk_cloudfront::types::builders::DistributionConfigBuilder;
use aws_sdk_cloudfront::types::builders::AliasesBuilder;
use aws_sdk_cloudfront::types::{Origin, Origins};
use aws_sdk_cloudfront::types::builders::{S3OriginConfigBuilder, OriginBuilder};
use aws_sdk_cloudfront::types::CloudFrontOriginAccessIdentityConfig;
use aws_sdk_cloudfront::types::builders::CloudFrontOriginAccessIdentityConfigBuilder;
use aws_sdk_cloudfront::types::builders::OriginsBuilder;
use std::collections::HashMap;
use authorizer::Auth;


pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(&shared_config)
}

fn make_aliases(domains: Vec<String>) -> Aliases {
    let it = AliasesBuilder::default();
    it
        .quantity(domains.len().try_into().unwrap())
        .set_items(Some(domains))
        .build()
        .unwrap()
}

fn make_origin(id: &str, origin_domain: &str, oai_id: &str) -> Origin {
    let oai_path = format!("origin-access-identity/cloudfront/{}", oai_id);
    let s3b = S3OriginConfigBuilder::default();

    let s3_origin_config = s3b
        .origin_access_identity(oai_path)
        .build();

    let it = OriginBuilder::default();
    it
        .id(id)
        .domain_name(origin_domain)
        .s3_origin_config(s3_origin_config)
        .build()
        .unwrap()
}

fn make_origins(origin_domain: &str, origin_paths: Vec<String>, oai_id: &str) -> Origins {
    let it = OriginsBuilder::default();
    let mut items: Vec<Origin> = vec![];
    for path in origin_paths {
        let origin = make_origin(&path, origin_domain, oai_id);
        items.push(origin);
    }
    it
        .quantity(items.len().try_into().unwrap())
        .set_items(Some(items))
        .build()
        .unwrap()
}

pub fn make_dist_config(
    caller_ref: &str,
    origin_domain: &str,
    origin_paths: Vec<String>,
    domains: Vec<String>,
    oai_id: &str

) -> DistributionConfig {
    let it = DistributionConfigBuilder::default();
    let origins = make_origins(origin_domain, origin_paths, oai_id);
    let aliases = make_aliases(domains);
    it
        .caller_reference(caller_ref)
        .aliases(aliases)
        .origins(origins)
        .build()
        .unwrap()
}

async fn list_distributions(client: &Client) -> HashMap<String, String> {
    let res = client
        .list_distributions()
        .send()
        .await
        .unwrap();
    let items = res.distribution_list.unwrap().items.unwrap();

    let mut h: HashMap<String, String> = HashMap::new();
    for item in items {
        let origins = item.origins.unwrap().items;
        for origin in origins {
            h.insert(origin.domain_name, item.id.clone());
        }
    }
    h
}

async fn find_distribution(client: &Client, domain: &str) -> Option<String> {
    let dists = list_distributions(client).await;
    dists.get(domain).cloned()
}

async fn update_distribution(
    client: &Client,
    id: &str,
    dc: DistributionConfig
) {
    let _ = client
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
        .await
        .unwrap();
    println!("{:?}", res);
}

pub async fn create_or_update_distribution(
    client: &Client,
    origin_domain: &str,
    dc: DistributionConfig
) {
    let maybe_dist = find_distribution(client, origin_domain).await;
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
    it.caller_reference(caller_ref).comment(comment).build().unwrap()
}

pub async fn create_oai(client: &Client, caller_ref: &str) -> String {
    let cfg = make_oai_config(caller_ref, "");

    let res = client
        .create_cloud_front_origin_access_identity()
        .cloud_front_origin_access_identity_config(cfg)
        .send()
        .await
        .unwrap();
    res.cloud_front_origin_access_identity.unwrap().id
}


// origin access control



pub async fn create_oac(client: &Client, name: &str) {

}
