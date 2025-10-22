use crate::Auth;
use aws_sdk_cloudfront::{
    Client,
    primitives::Blob,
    types::{
        Aliases,
        AllowedMethods,
        CacheBehaviors,
        CachePolicyConfig,
        CachePolicyType,
        CachedMethods,
        CustomErrorResponses,
        CustomHeaders,
        DefaultCacheBehavior,
        DistributionConfig,
        EventType,
        FunctionAssociation,
        FunctionAssociations,
        FunctionRuntime,
        GeoRestrictionType,
        HttpVersion,
        InvalidationBatch,
        LambdaFunctionAssociations,
        LoggingConfig,
        Method,
        MinimumProtocolVersion,
        Origin,
        OriginAccessControlConfig,
        OriginAccessControlOriginTypes,
        OriginAccessControlSigningBehaviors,
        OriginAccessControlSigningProtocols,
        Origins,
        Paths,
        PriceClass,
        Restrictions,
        SslSupportMethod,
        ViewerCertificate,
        ViewerProtocolPolicy,
        builders::{
            AliasesBuilder,
            AllowedMethodsBuilder,
            CacheBehaviorsBuilder,
            CachePolicyConfigBuilder,
            CachedMethodsBuilder,
            CustomErrorResponsesBuilder,
            CustomHeadersBuilder,
            DefaultCacheBehaviorBuilder,
            DistributionConfigBuilder,
            FunctionAssociationBuilder,
            FunctionAssociationsBuilder,
            FunctionConfigBuilder,
            GeoRestrictionBuilder,
            InvalidationBatchBuilder,
            LambdaFunctionAssociationsBuilder,
            LoggingConfigBuilder,
            OriginAccessControlConfigBuilder,
            OriginBuilder,
            OriginsBuilder,
            PathsBuilder,
            RestrictionsBuilder,
            S3OriginConfigBuilder,
            ViewerCertificateBuilder,
        },
    },
};
use kit as u;
use kit::LogUpdate;
use std::{
    collections::HashMap,
    io::stdout,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.get_global_config().await;
    Client::new(&shared_config)
}

async fn _get_distribution(client: &Client, dist_id: &str) -> DistributionConfig {
    let res = client.get_distribution().id(dist_id).send().await.unwrap();
    res.distribution.unwrap().distribution_config.unwrap()
}

fn make_custom_headers() -> CustomHeaders {
    let it = CustomHeadersBuilder::default();
    it.quantity(0).build().unwrap()
}

fn make_origin(id: &str, path: &str, origin_domain: &str, oac_id: &str) -> Origin {
    let s3b = S3OriginConfigBuilder::default();
    let s3config = s3b.origin_access_identity("").build();

    let custom_headers = make_custom_headers();

    let it = OriginBuilder::default();
    it.id(id)
        .domain_name(origin_domain)
        .origin_access_control_id(oac_id)
        .origin_path(path)
        .custom_headers(custom_headers)
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

fn make_cached_methods() -> CachedMethods {
    let it = CachedMethodsBuilder::default();
    let cached_methods = vec![Method::Get, Method::Head];
    it.quantity(2)
        .set_items(Some(cached_methods))
        .build()
        .unwrap()
}

fn make_allowed_methods() -> AllowedMethods {
    let it = AllowedMethodsBuilder::default();
    let methods = vec![Method::Get, Method::Head, Method::Options];
    let cached_methods = make_cached_methods();
    it.quantity(3)
        .set_items(Some(methods))
        .cached_methods(cached_methods)
        .build()
        .unwrap()
}

fn make_lambda_function_associations() -> LambdaFunctionAssociations {
    let it = LambdaFunctionAssociationsBuilder::default();
    it.quantity(0).build().unwrap()
}

fn make_function_assoc(arn: &str) -> FunctionAssociation {
    let it = FunctionAssociationBuilder::default();
    it.function_arn(arn)
        .event_type(EventType::ViewerRequest)
        .build()
        .unwrap()
}

fn make_function_associations(functions: Vec<String>) -> FunctionAssociations {
    let it = FunctionAssociationsBuilder::default();
    let len = functions.len().try_into().unwrap();
    if len == 0 {
        it.quantity(0).build().unwrap()
    } else {
        let mut assocs: Vec<FunctionAssociation> = vec![];
        for arn in functions {
            assocs.push(make_function_assoc(&arn));
        }
        it.quantity(len).set_items(Some(assocs)).build().unwrap()
    }
}

fn make_default_cache_behavior(
    origin_id: &str,
    cache_policy_id: &str,
    functions: Vec<String>,
) -> DefaultCacheBehavior {
    let allowed_methods = make_allowed_methods();
    let lambda_function_assocs = make_lambda_function_associations();
    let function_assocs = make_function_associations(functions);
    let it = DefaultCacheBehaviorBuilder::default();
    it.target_origin_id(origin_id)
        .viewer_protocol_policy(ViewerProtocolPolicy::RedirectToHttps)
        .allowed_methods(allowed_methods)
        .cache_policy_id(cache_policy_id)
        .smooth_streaming(false)
        .compress(false)
        .field_level_encryption_id("")
        .lambda_function_associations(lambda_function_assocs)
        .function_associations(function_assocs)
        .build()
        .unwrap()
}

fn make_cache_behaviors() -> CacheBehaviors {
    let it = CacheBehaviorsBuilder::default();
    it.quantity(0).build().unwrap()
}

fn make_custom_error_responses() -> CustomErrorResponses {
    let it = CustomErrorResponsesBuilder::default();
    it.quantity(0).build().unwrap()
}

fn make_geo_restrictions() -> Restrictions {
    let geo_res = GeoRestrictionBuilder::default()
        .restriction_type(GeoRestrictionType::None)
        .quantity(0)
        .build()
        .unwrap();
    let it = RestrictionsBuilder::default();
    it.geo_restriction(geo_res).build()
}

fn make_logging_config() -> LoggingConfig {
    let it = LoggingConfigBuilder::default();
    it.enabled(false).build()
}

fn make_aliases(alias: Option<String>) -> Aliases {
    let it = AliasesBuilder::default();
    let domains = match alias {
        Some(a) => vec![a],
        None => vec![],
    };
    it.quantity(domains.len().try_into().unwrap())
        .set_items(Some(domains))
        .build()
        .unwrap()
}

fn make_viewer_cert(maybe_cert_arn: Option<String>) -> ViewerCertificate {
    let it = ViewerCertificateBuilder::default();
    match maybe_cert_arn {
        Some(arn) => it
            .acm_certificate_arn(arn)
            .ssl_support_method(SslSupportMethod::SniOnly)
            .minimum_protocol_version(MinimumProtocolVersion::TlSv122019)
            .build(),
        None => it
            .cloud_front_default_certificate(true)
            .ssl_support_method(SslSupportMethod::SniOnly)
            .minimum_protocol_version(MinimumProtocolVersion::TlSv122019)
            .build(),
    }
}

pub fn make_dist_config(
    name: &str,
    default_root_object: &str,
    caller_ref: &str,
    origin_domain: &str,
    origin_paths: HashMap<String, String>,
    alias: Option<String>,
    cert_arn: Option<String>,
    oac_id: &str,
    cache_policy_id: &str,
    functions: Vec<String>,
) -> DistributionConfig {
    let it = DistributionConfigBuilder::default();
    let origins = make_origins(origin_domain, origin_paths, oac_id);
    let aliases = make_aliases(alias);
    let cert = make_viewer_cert(cert_arn);
    let default_origin_id = origins.items.first().unwrap().id.clone();
    let default_cache = make_default_cache_behavior(&default_origin_id, cache_policy_id, functions);
    let cache_behaviors = make_cache_behaviors();
    let custom_error_responses = make_custom_error_responses();
    let restrictions = make_geo_restrictions();
    let logging = make_logging_config();

    //viewer_certificate is None if domain is none

    it.caller_reference(caller_ref)
        .aliases(aliases)
        .viewer_certificate(cert)
        .origins(origins)
        .default_cache_behavior(default_cache)
        .cache_behaviors(cache_behaviors)
        .custom_error_responses(custom_error_responses)
        .restrictions(restrictions)
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

async fn update_distribution(
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
        .await;
    match res {
        Ok(_) => id.to_string(),
        Err(_) => id.to_string(),
    }
}

async fn get_status(client: &Client, dist_id: &str) -> String {
    let res = client.get_distribution().id(dist_id).send().await;
    match res {
        Ok(r) => r.distribution.unwrap().status,
        Err(_) => String::from(""),
    }
}

pub async fn wait_until_updated(client: &Client, dist_id: &str) {
    let mut log_update = LogUpdate::new(stdout()).unwrap();
    let mut status: String = get_status(client, dist_id).await;
    let _ = log_update.render(&format!("Waiting for distribution update: {}", &status));
    while status != "Deployed" || status.is_empty() {
        u::sleep(10000);
        let _ = log_update.render(&format!("Waiting for distribution update: {}", &status));
        status = get_status(client, dist_id).await;
    }
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
    let maybe_dist = find_distribution(client, name).await;
    match maybe_dist {
        Some((id, e_tag)) => update_distribution(client, &id, &e_tag, dc).await,
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
pub async fn get_cname(client: &Client, dist_id: &str) -> String {
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

pub async fn assoc_alias(client: &Client, dist_id: &str, domain: &str) {
    client
        .associate_alias()
        .alias(domain)
        .target_distribution_id(dist_id)
        .send()
        .await
        .unwrap();
}

// function

pub async fn create_function(client: &Client, name: &str, handler: &str) -> String {
    let buffer = handler.as_bytes();
    let blob = Blob::new(buffer);
    let fcb = FunctionConfigBuilder::default();
    let fc = fcb
        .runtime(FunctionRuntime::CloudfrontJs20)
        .comment(name)
        .build()
        .unwrap();
    let res = client
        .create_function()
        .name(name)
        .function_config(fc)
        .function_code(blob)
        .send()
        .await
        .unwrap();
    res.e_tag.unwrap()
}

pub async fn update_function(client: &Client, name: &str, handler: &str, etag: &str) -> String {
    let buffer = handler.as_bytes();
    let blob = Blob::new(buffer);
    let fcb = FunctionConfigBuilder::default();
    let fc = fcb
        .runtime(FunctionRuntime::CloudfrontJs20)
        .comment(name)
        .build()
        .unwrap();
    let res = client
        .update_function()
        .name(name)
        .function_config(fc)
        .function_code(blob)
        .if_match(etag)
        .send()
        .await
        .unwrap();
    res.e_tag.unwrap()
}

pub async fn publish_function(client: &Client, name: &str, etag: &str) {
    client
        .publish_function()
        .name(name)
        .if_match(etag)
        .send()
        .await
        .unwrap();
}

pub async fn find_function(client: &Client, name: &str) -> Option<String> {
    let res = client.get_function().name(name).send().await;

    match res {
        Ok(r) => r.e_tag,
        Err(_) => None,
    }
}

pub async fn create_or_update_function(client: &Client, name: &str, handler: &str) {
    let maybe_fn = find_function(client, name).await;
    match maybe_fn {
        Some(_) => (),
        None => {
            let etag = create_function(client, name, handler).await;
            publish_function(client, name, &etag).await;
        }
    };
}

pub async fn delete_distribution(client: &Client, name: &str) {
    let maybe_dist = find_distribution(client, name).await;
    match maybe_dist {
        Some((id, e_tag)) => {
            client
                .delete_distribution()
                .id(id)
                .if_match(e_tag)
                .send()
                .await
                .unwrap();
        }
        None => println!("CF distribution not found, skipping"),
    }
}
