use crate::Auth;
use aws_sdk_cloudfront::{
    Client,
    types::{
        Aliases,
        AllowedMethods,
        CachePolicyConfig,
        CachePolicyType,
        DefaultCacheBehavior,
        DistributionConfig,
        HttpVersion,
        InvalidationBatch,
        LoggingConfig,
        Method,
        Origin,
        OriginAccessControlConfig,
        OriginAccessControlOriginTypes,
        OriginAccessControlSigningBehaviors,
        OriginAccessControlSigningProtocols,
        Origins,
        Paths,
        PriceClass,
        ViewerProtocolPolicy,
        builders::{
            AliasesBuilder,
            AllowedMethodsBuilder,
            CachePolicyConfigBuilder,
            DefaultCacheBehaviorBuilder,
            DistributionConfigBuilder,
            InvalidationBatchBuilder,
            LoggingConfigBuilder,
            OriginAccessControlConfigBuilder,
            OriginBuilder,
            OriginsBuilder,
            PathsBuilder,
            S3OriginConfigBuilder,
        },
    },
};
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(&shared_config)
}

fn make_aliases(domains: Vec<String>) -> Aliases {
    let it = AliasesBuilder::default();
    it.quantity(domains.len().try_into().unwrap())
        .set_items(Some(domains))
        .build()
        .unwrap()
}

async fn _get_distribution(client: &Client, dist_id: &str) -> DistributionConfig {
    let res = client.get_distribution().id(dist_id).send().await.unwrap();
    res.distribution.unwrap().distribution_config.unwrap()
}

fn make_origin(id: &str, path: &str, origin_domain: &str, oac_id: &str) -> Origin {
    let s3b = S3OriginConfigBuilder::default();
    let s3config = s3b.origin_access_identity("").build();

    let it = OriginBuilder::default();
    it.id(id)
        .domain_name(origin_domain)
        .origin_access_control_id(oac_id)
        .origin_path(path)
        .s3_origin_config(s3config)
        .build()
        .unwrap()
}

fn make_origins(
    origin_domain: &str,
    origin_paths: HashMap<String, String>,
    oac_id: &str,
) -> Origins {
    let it = OriginsBuilder::default();
    let mut items: Vec<Origin> = vec![];

    for (id, path) in origin_paths {
        let origin = make_origin(&id, &path, origin_domain, oac_id);
        items.push(origin);
    }
    it.quantity(items.len().try_into().unwrap())
        .set_items(Some(items))
        .build()
        .unwrap()
}

fn make_allowed_methods() -> AllowedMethods {
    let it = AllowedMethodsBuilder::default();
    let methods = vec![Method::Get, Method::Head, Method::Options];
    it.quantity(3).set_items(Some(methods)).build().unwrap()
}

fn make_default_cache_behavior(origin_id: &str, cache_policy_id: &str) -> DefaultCacheBehavior {
    let allowed_methods = make_allowed_methods();
    let it = DefaultCacheBehaviorBuilder::default();
    it.target_origin_id(origin_id)
        .viewer_protocol_policy(ViewerProtocolPolicy::RedirectToHttps)
        .allowed_methods(allowed_methods)
        .cache_policy_id(cache_policy_id)
        .build()
        .unwrap()
}

fn make_logging_config() -> LoggingConfig {
    let it = LoggingConfigBuilder::default();
    it.enabled(false).build()
}

pub fn make_dist_config(
    name: &str,
    default_root_object: &str,
    caller_ref: &str,
    origin_domain: &str,
    origin_paths: HashMap<String, String>,
    domains: Vec<String>,
    oac_id: &str,
    cache_policy_id: &str,
) -> DistributionConfig {
    let it = DistributionConfigBuilder::default();
    let origins = make_origins(origin_domain, origin_paths, oac_id);
    let aliases = make_aliases(domains);
    let default_origin_id = origins.items.first().unwrap().id.clone();
    let default_cache = make_default_cache_behavior(&default_origin_id, cache_policy_id);
    let logging = make_logging_config();

    it.caller_reference(caller_ref)
        .aliases(aliases)
        .origins(origins)
        .default_cache_behavior(default_cache)
        .price_class(PriceClass::PriceClass100)
        .logging(logging)
        .default_root_object(default_root_object)
        .web_acl_id("")
        .http_version(HttpVersion::Http2)
        .comment(name)
        .enabled(true)
        .build()
        .unwrap()
}

async fn list_distributions(client: &Client) -> HashMap<String, (String, String)> {
    let res = client.list_distributions().send().await.unwrap();

    let xs = res.distribution_list;
    let mut h: HashMap<String, (String, String)> = HashMap::new();

    if let Some(m) = xs {
        match m.items {
            Some(xs) => {
                for x in xs {
                    let e_tag = x.e_tag.unwrap();
                    let id = x.id;
                    h.insert(x.comment.clone(), (id.clone(), e_tag.clone()));
                }
            }
            None => (),
        }
    }
    h
}

pub async fn find_distribution(client: &Client, name: &str) -> Option<(String, String)> {
    let dists = list_distributions(client).await;
    dists.get(name).cloned()
}

async fn _update_distribution(
    client: &Client,
    id: &str,
    e_tag: &str,
    dc: DistributionConfig,
) -> String {
    let res = client
        .update_distribution()
        .id(id)
        .distribution_config(dc)
        .if_match(e_tag)
        .send()
        .await
        .unwrap();

    res.e_tag.unwrap()
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
    name: &str,
    dc: DistributionConfig,
) -> String {
    //update_distribution(client, &id, &e_tag, dc).await,
    let maybe_dist = find_distribution(client, name).await;
    match maybe_dist {
        Some((id, _e_tag)) => id,
        None => create_distribution(client, dc).await,
    }
}

// cache policy

async fn list_cache_policies(client: &Client) -> HashMap<String, String> {
    let res = client
        .list_cache_policies()
        .set_type(Some(CachePolicyType::Custom))
        .send()
        .await
        .unwrap();
    let mut h: HashMap<String, String> = HashMap::new();
    let items = res.cache_policy_list.unwrap().items;
    if let Some(item) = items {
        for x in item {
            let cp = x.cache_policy.unwrap();
            let name = cp.cache_policy_config.unwrap().name;
            h.insert(name, cp.id);
        }
    }
    h
}

async fn find_cache_policy(client: &Client, name: &str) -> Option<String> {
    let h = list_cache_policies(client).await;
    h.get(name).cloned()
}

fn make_cache_policy_config(name: &str) -> CachePolicyConfig {
    let it = CachePolicyConfigBuilder::default();
    it.name(name).min_ttl(60).build().unwrap()
}

async fn create_cache_policy(client: &Client, name: &str) -> String {
    let cfg = make_cache_policy_config(name);
    let res = client
        .create_cache_policy()
        .cache_policy_config(cfg)
        .send()
        .await
        .unwrap();
    res.cache_policy.unwrap().id
}

pub async fn find_or_create_cache_policy(client: &Client, name: &str) -> String {
    let maybe_id = find_cache_policy(client, name).await;
    match maybe_id {
        Some(id) => id,
        None => create_cache_policy(client, name).await,
    }
}

// origin access control

async fn list_oacs(client: &Client) -> HashMap<String, String> {
    let res = client.list_origin_access_controls().send().await.unwrap();
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
    let h = list_oacs(client).await;
    h.get(origin_domain).cloned()
}

fn make_oac_config(name: &str) -> OriginAccessControlConfig {
    let it = OriginAccessControlConfigBuilder::default();
    it.name(name)
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
        None => create_oac(client, origin_domain).await,
    }
}

// get domain
pub async fn get_url(client: &Client, dist_id: &str) -> String {
    let res = client.get_distribution().id(dist_id).send().await.unwrap();
    res.distribution.unwrap().domain_name
}

// invalidations

fn make_paths() -> Paths {
    let it = PathsBuilder::default();
    let items = vec![String::from("/*")];
    it.quantity(1).set_items(Some(items)).build().unwrap()
}

fn make_invalidation_batch(caller_ref: &str) -> InvalidationBatch {
    let it = InvalidationBatchBuilder::default();
    let paths = make_paths();
    it.paths(paths)
        .caller_reference(caller_ref)
        .build()
        .unwrap()
}

pub async fn create_invalidation(client: &Client, dist_id: &str) {
    let caller_ref = kit::utc_now();
    let invalidation_batch = make_invalidation_batch(&caller_ref);
    let _ = client
        .create_invalidation()
        .distribution_id(dist_id)
        .invalidation_batch(invalidation_batch)
        .send()
        .await
        .unwrap();
}

// function

// associate
