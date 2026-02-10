use crate::Auth;
pub use aws_sdk_apigatewayv2::Client;
use aws_sdk_apigatewayv2::{
    Error,
    types::{
        AuthorizationType,
        AuthorizerType,
        Cors,
        JwtConfiguration,
        ProtocolType,
        EndpointType,
        DomainNameConfiguration,
        RouteSettings,
        builders::{
            CorsBuilder,
            JwtConfigurationBuilder,
            DomainNameConfigurationBuilder,
            RouteSettingsBuilder
        },
    },
};
use colored::Colorize;
use kit::*;
use std::collections::HashMap;

mod eventbridge;
mod lambda;
mod sfn;
mod sqs;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub fn make_cors(methods: Vec<String>, origins: Vec<String>, headers: Option<Vec<String>>) -> Cors {
    let f = CorsBuilder::default();
    f.set_allow_methods(Some(methods))
        .set_allow_origins(Some(origins))
        .set_allow_headers(headers)
        .build()
}

pub async fn create_api(
    client: &Client,
    name: &str,
    cors: Option<Cors>,
    tags: HashMap<String, String>,
) -> String {
    let r = client
        .create_api()
        .name(name)
        .protocol_type(ProtocolType::Http)
        .set_cors_configuration(cors)
        .set_tags(Some(tags))
        .send()
        .await
        .unwrap();
    r.api_id.unwrap()
}

pub async fn delete_api(client: &Client, api_id: &str) {
    client.delete_api().api_id(api_id).send().await.unwrap();
}

pub async fn find_api(client: &Client, name: &str) -> Option<String> {
    let r = client
        .get_apis()
        .max_results(s!("1000"))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(apis) => {
            for api in apis.to_vec() {
                match api.name {
                    Some(nm) => {
                        if nm == name {
                            return api.api_id;
                        }
                    }
                    None => (),
                }
            }
            return None;
        }
        None => None,
    }
}

pub async fn update_api(
    client: &Client,
    name: &str,
    api_id: &str,
    cors: Option<Cors>,
    _tags: HashMap<String, String>,
) -> String {
    println!("Updating route {} (cors)", name);
    let _ = client
        .update_api()
        .api_id(s!(api_id))
        .set_cors_configuration(cors)
        .send()
        .await
        .unwrap();
    s!(api_id)
}

pub async fn create_or_update_api(
    client: &Client,
    name: &str,
    cors: Option<Cors>,
    tags: HashMap<String, String>,
) -> String {
    let api_id = find_api(client, name).await;
    match api_id {
        Some(id) => {
            tracing::debug!("Found API {} ({})", name.green(), &id);
            update_api(client, name, &id, cors, tags).await
        }
        _ => {
            println!("Creating route {} (gateway)", name.blue());
            create_api(client, name, cors, tags).await
        }
    }
}

pub async fn find_route(client: &Client, api_id: &str, route_key: &str) -> Option<String> {
    let r = client
        .get_routes()
        .api_id(api_id.to_string())
        .max_results(s!("2000"))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(routes) => {
            for route in routes.to_vec() {
                match route.route_key {
                    Some(key) => {
                        if &key == route_key {
                            return route.route_id;
                        }
                    }
                    None => (),
                }
            }
            return None;
        }
        None => None,
    }
}

async fn create_route(
    client: &Client,
    api_id: &str,
    route_key: &str,
    target: &str,
    authorizer: Option<String>,
    auth_kind: AuthorizationType,
) -> String {
    match authorizer {
        Some(auth) => {
            let res = client
                .create_route()
                .api_id(s!(api_id))
                .route_key(route_key)
                .target(target)
                .authorization_type(auth_kind)
                .authorizer_id(auth)
                .send()
                .await
                .unwrap();
            res.route_id.unwrap()
        }
        _ => {
            let res = client
                .create_route()
                .api_id(s!(api_id))
                .route_key(route_key)
                .target(target)
                .send()
                .await
                .unwrap();
            res.route_id.unwrap()
        }
    }
}

pub async fn update_route(
    client: &Client,
    api_id: &str,
    route_id: &str,
    target: &str,
    authorizer: Option<String>,
    auth_kind: AuthorizationType,
) -> String {
    match authorizer {
        Some(auth) => {
            let res = client
                .update_route()
                .api_id(s!(api_id))
                .route_id(route_id)
                .target(target)
                .authorization_type(auth_kind)
                .authorizer_id(auth)
                .send()
                .await
                .unwrap();
            res.route_id.unwrap()
        }
        _ => {
            let res = client
                .update_route()
                .api_id(s!(api_id))
                .route_id(route_id)
                .target(target)
                .send()
                .await
                .unwrap();
            res.route_id.unwrap()
        }
    }
}

pub async fn create_or_update_route(
    client: &Client,
    api_id: &str,
    method: &str,
    path: &str,
    integration_id: &str,
    authorizer_id: Option<String>,
    authorizer_kind: &str,
) {
    let route_key = strip(&format!("{} {}", method, path), "/");
    let target = format!("integrations/{}", integration_id);
    let maybe_route = find_route(client, api_id, &route_key).await;

    let auth_kind = match authorizer_kind {
        "cognito" => AuthorizationType::Jwt,
        _ => AuthorizationType::Custom,
    };

    match maybe_route {
        Some(route_id) => {
            println!("Updating route {} ({})", &route_key.green(), &route_id);
            update_route(client, api_id, &route_id, &target, authorizer_id, auth_kind).await;
        }
        None => {
            println!("Creating route {}", &route_key.blue());
            create_route(
                client,
                api_id,
                &route_key,
                &target,
                authorizer_id,
                auth_kind,
            )
            .await;
        }
    }
}

pub async fn create_deployment(client: &Client, api_id: &str, stage: &str) {
    client
        .create_deployment()
        .api_id(api_id)
        .stage_name(stage)
        .send()
        .await
        .unwrap();
}

pub async fn delete_route(client: &Client, api_id: &str, route_id: &str) -> Result<(), Error> {
    let res = client
        .delete_route()
        .api_id(api_id)
        .route_id(route_id)
        .send()
        .await;

    match res {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

pub async fn find_authorizer(
    client: &Client,
    api_id: &str,
    authorizer_name: &str,
) -> Option<String> {
    let r = client
        .get_authorizers()
        .api_id(s!(api_id))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(auths) => {
            for auth in auths.to_vec() {
                match auth.name {
                    Some(name) => {
                        if name == authorizer_name {
                            return auth.authorizer_id;
                        }
                    }
                    None => (),
                }
            }
            return None;
        }
        None => None,
    }
}

pub async fn create_lambda_authorizer(
    client: &Client,
    api_id: &str,
    name: &str,
    uri: &str,
) -> String {
    println!("Creating authorizer: {}", name.blue());
    let res = client
        .create_authorizer()
        .name(s!(name))
        .api_id(s!(api_id))
        .authorizer_type(AuthorizerType::Request)
        .authorizer_uri(s!(uri))
        .authorizer_result_ttl_in_seconds(0)
        .authorizer_payload_format_version(s!("2.0"))
        .identity_source(s!("$request.header.Authorization"))
        .send()
        .await;
    res.unwrap().authorizer_id.unwrap()
}

pub async fn update_lambda_authorizer(
    client: &Client,
    id: &str,
    api_id: &str,
    uri: &str,
) -> String {
    let res = client
        .update_authorizer()
        .authorizer_id(s!(id))
        .api_id(s!(api_id))
        .authorizer_type(AuthorizerType::Request)
        .authorizer_uri(s!(uri))
        .authorizer_payload_format_version(s!("2.0"))
        .authorizer_result_ttl_in_seconds(0)
        .identity_source(s!("$request.header.Authorization"))
        .send()
        .await;
    res.unwrap().authorizer_id.unwrap()
}

pub async fn create_or_update_lambda_authorizer(
    client: &Client,
    api_id: &str,
    name: &str,
    uri: &str,
) -> String {
    let maybe_authorizer_id = find_authorizer(client, api_id, name).await;
    match maybe_authorizer_id {
        Some(id) => {
            println!("Updating authorizer {}", name.green());
            update_lambda_authorizer(client, &id, api_id, uri).await
        }
        None => create_lambda_authorizer(client, api_id, name, uri).await,
    }
}

// jwt
fn make_jwt_config(issuer: &str, client_id: &str) -> JwtConfiguration {
    let f = JwtConfigurationBuilder::default();
    f.audience(client_id).issuer(issuer).build()
}

pub async fn create_cognito_authorizer(
    client: &Client,
    api_id: &str,
    name: &str,
    jwt_config: JwtConfiguration,
) -> String {
    println!("Creating authorizer: {}", name.blue());
    let res = client
        .create_authorizer()
        .name(s!(name))
        .api_id(s!(api_id))
        .authorizer_type(AuthorizerType::Jwt)
        .jwt_configuration(jwt_config)
        .authorizer_result_ttl_in_seconds(0)
        .identity_source(s!("$request.header.Authorization"))
        .send()
        .await;
    res.unwrap().authorizer_id.unwrap()
}

pub async fn update_cognito_authorizer(
    client: &Client,
    id: &str,
    api_id: &str,
    jwt_config: JwtConfiguration,
) -> String {
    let res = client
        .update_authorizer()
        .authorizer_id(s!(id))
        .api_id(s!(api_id))
        .authorizer_type(AuthorizerType::Jwt)
        .jwt_configuration(jwt_config)
        .authorizer_result_ttl_in_seconds(0)
        .identity_source(s!("$request.header.Authorization"))
        .send()
        .await;
    res.unwrap().authorizer_id.unwrap()
}

pub async fn create_or_update_cognito_authorizer(
    client: &Client,
    api_id: &str,
    name: &str,
    issuer: &str,
    client_id: &str,
) -> String {
    let jwt_config = make_jwt_config(issuer, client_id);
    let maybe_authorizer_id = find_authorizer(client, api_id, name).await;
    match maybe_authorizer_id {
        Some(id) => {
            println!("Updating authorizer {}", name.green());
            update_cognito_authorizer(client, &id, api_id, jwt_config).await
        }
        None => create_cognito_authorizer(client, api_id, name, jwt_config).await,
    }
}

pub async fn delete_authorizer(client: &Client, api_id: &str, name: &str) {
    let maybe_authorizer_id = find_authorizer(client, api_id, name).await;
    match maybe_authorizer_id {
        Some(id) => {
            println!("Deleting authorizer {} ({})", name.green(), &id);
            let _ = client
                .delete_authorizer()
                .authorizer_id(s!(id))
                .api_id(s!(api_id))
                .authorizer_id(s!(id))
                .send()
                .await;
        }
        None => (),
    }
}

fn make_route_settings(burst_limit: Option<i32>, rate_limit: Option<f64>) -> RouteSettings {
    let f = RouteSettingsBuilder::default();
    f.set_throttling_burst_limit(burst_limit)
        .set_throttling_rate_limit(rate_limit)
        .build()
}

async fn _create_stage(
    client: &Client,
    api_id: &str,
    stage: &str,
    burst_limit: Option<i32>,
    rate_limit: Option<f64>
) {
    let route_settings = make_route_settings(burst_limit, rate_limit);
    tracing::debug!("Creating stage {}", &stage.green());
    let _ = client
        .create_stage()
        .api_id(s!(api_id))
        .auto_deploy(true)
        .stage_name(stage)
        .default_route_settings(route_settings)
        .send()
        .await;
}

async fn update_stage(
    client: &Client,
    api_id: &str,
    stage: &str,
    burst_limit: Option<i32>,
    rate_limit: Option<f64>
) {
    let route_settings = make_route_settings(burst_limit, rate_limit);
        let _ = client
        .update_stage()
        .api_id(s!(api_id))
        .auto_deploy(true)
        .stage_name(stage)
        .default_route_settings(route_settings)
        .send()
        .await;

}

pub async fn create_or_update_stage(
    client: &Client,
    api_id: &str,
    stage: &str,
    burst_limit: Option<i32>,
    rate_limit: Option<f64>
) {
    update_stage(client, api_id, stage, burst_limit, rate_limit).await;
}

pub async fn create_lambda_integration(
    client: &Client,
    api_id: &str,
    target_arn: &str,
    role: &str,
    is_async: bool,
) -> String {
    lambda::create_or_update(client, api_id, target_arn, role, is_async).await
}

pub async fn create_sfn_integration(
    client: &Client,
    api_id: &str,
    name: &str,
    role: &str,
    is_async: bool,
    request_params: HashMap<String, String>,
) -> String {
    sfn::find_or_create(client, api_id, role, request_params, is_async, name).await
}

pub async fn create_event_integration(
    client: &Client,
    api_id: &str,
    name: &str,
    role: &str,
    request_params: HashMap<String, String>,
) -> String {
    eventbridge::find_or_create(client, api_id, role, request_params, name).await
}

pub async fn create_sqs_integration(
    client: &Client,
    api_id: &str,
    name: &str,
    role: &str,
    request_params: HashMap<String, String>,
) -> String {
    sqs::find_or_create(client, api_id, role, request_params, name).await
}

pub async fn delete_lambda_integration(client: &Client, api_id: &str, target_arn: &str) {
    lambda::delete(client, api_id, target_arn).await
}

pub async fn delete_sfn_integration(client: &Client, api_id: &str, name: &str) {
    sfn::delete(client, api_id, name).await
}

pub async fn delete_event_integration(client: &Client, api_id: &str, name: &str) {
    eventbridge::delete(client, api_id, name).await
}

pub async fn delete_sqs_integration(client: &Client, api_id: &str, name: &str) {
    sqs::delete(client, api_id, name).await
}

pub async fn find_api_id(client: &Client, name: &str) -> Option<String> {
    let r = client
        .get_apis()
        .max_results(String::from("1000"))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(apis) => {
            for api in apis.to_vec() {
                match api.name {
                    Some(n) => {
                        if n == name {
                            return api.api_id;
                        }
                    }
                    None => (),
                }
            }
            return None;
        }
        None => None,
    }
}

async fn list_apis_by_token(
    client: &Client,
    token: &str,
) -> (HashMap<String, HashMap<String, String>>, Option<String>) {
    let res = client
        .get_apis()
        .next_token(token.to_string())
        .send()
        .await
        .unwrap();
    let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
    let apis = res.items.unwrap();
    for api in apis {
        h.insert(api.name.unwrap(), api.tags.unwrap());
    }
    (h, res.next_token)
}

async fn list_apis(client: &Client) -> HashMap<String, HashMap<String, String>> {
    let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
    let r = client.get_apis().send().await;
    match r {
        Ok(res) => {
            let mut token: Option<String> = res.next_token;

            let apis = res.items.unwrap();
            for api in apis {
                h.insert(api.name.unwrap(), api.tags.unwrap());
            }

            match token {
                Some(tk) => {
                    token = Some(tk);
                    while token.is_some() {
                        let (xs, t) = list_apis_by_token(client, &token.unwrap()).await;
                        h.extend(xs.clone());
                        token = t.clone();
                        if let Some(x) = t {
                            if x.is_empty() {
                                break;
                            }
                        }
                    }
                }
                None => (),
            }
        }
        Err(e) => panic!("{}", e),
    }
    h
}

pub async fn find_tags(client: &Client, name: &str) -> HashMap<String, String> {
    let apis = list_apis(client).await;
    let maybe_h = apis.get(name);
    match maybe_h {
        Some(p) => p.clone(),
        None => HashMap::new(),
    }
}

// domain

async fn find_domain(client: &Client, domain_name: &str) -> Option<String> {
    let res = client
        .get_domain_name()
        .domain_name(domain_name)
        .send()
        .await;
    match res {
        Ok(r) => {
            let cfgs = r.domain_name_configurations.unwrap();
            let cfg = cfgs.first();
            cfg.unwrap().api_gateway_domain_name.clone()
        },
        Err(_) => None
    }
}

async fn _update_domain(client: &Client, domain_name: &str, cfg: DomainNameConfiguration) {
    let _ = client
        .update_domain_name()
        .domain_name(domain_name)
        .domain_name_configurations(cfg)
        .send()
        .await;
}

async fn create_domain(client: &Client, domain_name: &str, cfg: DomainNameConfiguration) -> String {
    let res = client
        .create_domain_name()
        .domain_name(domain_name)
        .domain_name_configurations(cfg)
        .send()
        .await;
    let cfgs = res.unwrap().domain_name_configurations.unwrap();
    let cfg = cfgs.first();
    cfg.unwrap().api_gateway_domain_name.clone().unwrap()
}

fn make_domain_config(name: &str, cert_arn: &str) -> DomainNameConfiguration  {
    let f = DomainNameConfigurationBuilder::default();
    f.api_gateway_domain_name(name)
        .certificate_arn(cert_arn)
        .endpoint_type(EndpointType::Regional)
        .build()
}

pub async fn create_or_update_domain(
    client: &Client,
    api_id: &str,
    domain_name: &str,
    stage: &str,
    cert_arn: &str,
    _hosted_zone_id: &str
) -> String {
    let cfg = make_domain_config(domain_name, cert_arn);

    let maybe_domain = find_domain(client, domain_name).await;

    if let Some(d) = maybe_domain {
        let _ = client
            .create_api_mapping()
            .api_id(api_id)
            .domain_name(domain_name)
            .stage(stage)
            .send()
            .await;
        d
    } else {
        let d = create_domain(client, domain_name, cfg).await;
        let _ = client
            .create_api_mapping()
            .api_id(api_id)
            .domain_name(domain_name)
            .stage(stage)
            .send()
            .await;
        d
    }
}


pub type GatewayCors = aws_sdk_apigatewayv2::types::Cors;
