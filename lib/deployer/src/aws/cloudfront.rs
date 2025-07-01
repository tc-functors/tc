use aws_sdk_cloudfront::{Client};
use aws_sdk_cloudfront::types::{DistributionConfig, Aliases};
use aws_sdk_cloudfront::types::builders::DistributionConfigBuilder;
use aws_sdk_cloudfront::types::builders::AliasesBuilder;
use aws_sdk_cloudfront::types::{Origin, Origins};
use aws_sdk_cloudfront::types::builders::{S3OriginConfigBuilder, OriginBuilder};
use aws_sdk_cloudfront::types::builders::OriginsBuilder;
use aws_sdk_cloudfront::types::OriginAccessControlConfig;
use aws_sdk_cloudfront::types::builders::OriginAccessControlConfigBuilder;
use aws_sdk_cloudfront::types::OriginAccessControlSigningProtocols;
use aws_sdk_cloudfront::types::OriginAccessControlSigningBehaviors;
use aws_sdk_cloudfront::types::OriginAccessControlOriginTypes;
use aws_sdk_cloudfront::types::DefaultCacheBehavior;
use aws_sdk_cloudfront::types::builders::DefaultCacheBehaviorBuilder;
use aws_sdk_cloudfront::types::AllowedMethods;
use aws_sdk_cloudfront::types::builders::AllowedMethodsBuilder;
use aws_sdk_cloudfront::types::CachedMethods;
use aws_sdk_cloudfront::types::Method;
use aws_sdk_cloudfront::types::ViewerProtocolPolicy;

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

fn make_origin(id: &str, origin_domain: &str, oac_id: &str) -> Origin {
    let it = OriginBuilder::default();
    it
        .id(id)
        .domain_name(origin_domain)
        .origin_access_control_id(oac_id)
        .build()
        .unwrap()
}

fn make_origins(origin_domain: &str, origin_paths: Vec<String>, oac_id: &str) -> Origins {
    let it = OriginsBuilder::default();
    let mut items: Vec<Origin> = vec![];
    for path in origin_paths {
        let origin = make_origin(&path, origin_domain, oac_id);
        items.push(origin);
    }
    it
        .quantity(items.len().try_into().unwrap())
        .set_items(Some(items))
        .build()
        .unwrap()
}


fn make_allowed_methods() -> AllowedMethods {
    let it = AllowedMethodsBuilder::default();
    let methods = vec![Method::Get, Method::Head, Method::Options];
    it
        .quantity(3)
        .set_items(Some(methods))
        .build()
        .unwrap()
}

fn make_default_cache_behavior(origin_id: &str) -> DefaultCacheBehavior {
    let allowed_methods = make_allowed_methods();
    let it = DefaultCacheBehaviorBuilder::default();
    it
        .target_origin_id(origin_id)
        .viewer_protocol_policy(ViewerProtocolPolicy::RedirectToHttps)
        .allowed_methods(allowed_methods)
        .build()
        .unwrap()

}

pub fn make_dist_config(
    caller_ref: &str,
    origin_domain: &str,
    origin_paths: Vec<String>,
    domains: Vec<String>,
    oac_id: &str

) -> DistributionConfig {
    let it = DistributionConfigBuilder::default();
    let origins = make_origins(origin_domain, origin_paths, oac_id);
    let aliases = make_aliases(domains);
    //let default_origin_id = origins.items.first().unwrap().id.clone();
    let default_cache = make_default_cache_behavior(oac_id);
    it
        .caller_reference(caller_ref)
        .aliases(aliases)
        .origins(origins)
        .default_cache_behavior(default_cache)
        .comment(caller_ref)
        .enabled(true)
        .build()
        .unwrap()
}

async fn list_distributions(client: &Client) -> HashMap<String, String> {
    let res = client
        .list_distributions()
        .send()
        .await
        .unwrap();


    let xs = res.distribution_list;
    let mut h: HashMap<String, String> = HashMap::new();

    if let Some(m) = xs {

       match m.items {
           Some(xs) =>  {
               for x in xs {
                   let origins = x.origins.unwrap().items;
                   for origin in origins {
                       h.insert(origin.domain_name, origin.id.clone());
                   }
               }
           },
           None => ()
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
) -> String {
    let res = client
        .update_distribution()
        .distribution_config(dc)
        .send()
        .await;
    id.to_string()
}

async fn create_distribution(client: &Client, dc: DistributionConfig) -> String {
    let res = client
        .create_distribution()
        .distribution_config(dc)
        .send()
        .await
        .unwrap();
    res.distribution.unwrap().id
}

pub async fn create_or_update_distribution(
    client: &Client,
    origin_domain: &str,
    dc: DistributionConfig
) -> String {

    let maybe_dist = find_distribution(client, origin_domain).await;
    match maybe_dist {
        Some(d) => update_distribution(client, &d, dc).await,
        None => create_distribution(client, dc).await
    }
}

pub async fn create_invalidation(client: &Client) {

}


// origin access control


async fn list_oacs(client: &Client) -> HashMap<String, String> {
    let res = client
        .list_origin_access_controls()
        .send()
        .await
        .unwrap();
    let mut h: HashMap<String, String> = HashMap::new();
    let items = res.origin_access_control_list.unwrap().items;
    if let Some(item) = items {
        for x in item {
            h.insert(x.name, x.id);
        }
    }

    h
}

async fn find_oac(client: &Client, origin_domain: &str) -> Option<String> {
    let h =  list_oacs(client).await;
    h.get(origin_domain).cloned()
}

fn make_oac_config(name: &str) -> OriginAccessControlConfig {
    let it = OriginAccessControlConfigBuilder::default();
    it
        .name(name)
        .signing_protocol(OriginAccessControlSigningProtocols::Sigv4)
        .signing_behavior(OriginAccessControlSigningBehaviors::Always)
        .origin_access_control_origin_type(OriginAccessControlOriginTypes::S3)
        .build()
        .unwrap()
}

async fn create_oac(client: &Client, origin_domain: &str) -> String {
    let cfg = make_oac_config(origin_domain);
    let res = client
        .create_origin_access_control()
        .origin_access_control_config(cfg)
        .send()
        .await
        .unwrap();
    res.origin_access_control.unwrap().id
}

pub async fn find_or_create_oac(client: &Client, origin_domain: &str) -> String {
    let maybe_oac = find_oac(client, origin_domain).await;
    match maybe_oac {
        Some(id) => id,
        None => create_oac(client, origin_domain).await
    }
}
