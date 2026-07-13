use compiler::Entity;
use composer::{
    Route,
    Throttling,
    aws::route::{
        Authorizer,
        Target,
    },
};
use itertools::Itertools;
use kit as u;
use kit::*;
use provider::{
    Auth,
    aws::{
        acm,
        cognito,
        gateway,
        gateway::{
            Client,
            GatewayCors as Cors,
        },
        lambda,
        lambda::LambdaClient,
        route53,
    },
};
use std::collections::HashMap;

async fn add_target_permission(auth: &Auth, api_id: &str, target: &Target) {
    let Target { entity, arn, .. } = target;
    match entity {
        Entity::Function => {
            let client = lambda::make_client(auth).await;
            let source_arn = auth.api_arn(api_id);
            let principal = "apigateway.amazonaws.com";
            let _ = lambda::add_permission(client, arn, principal, &source_arn, api_id).await;
        }
        _ => (),
    }
}

async fn find_alias_arn(client: &LambdaClient, arn: &str) -> String {
    let maybe_alias_arn = lambda::find_alias_arn(&client, arn).await;
    match maybe_alias_arn {
        Some(a) => {
            println!("Using alias function={}", &a);
            a
        }
        None => arn.to_string(),
    }
}

async fn create_integration(
    auth: &Auth,
    client: &Client,
    api_id: &str,
    route: &Route,
    target: &Target,
) -> String {
    let Target {
        entity,
        arn,
        request_params,
        ..
    } = target;

    let Route {
        role_arn,
        method,
        is_async,
        ..
    } = route;

    let int_name = format!("{}-{}", entity.to_str(), method);

    match entity {
        Entity::Function => {
            let lc = lambda::make_client(auth).await;
            let alias_arn = find_alias_arn(&lc, arn).await;
            gateway::create_lambda_integration(
                client,
                api_id,
                &alias_arn,
                role_arn,
                *is_async,
                request_params.clone(),
            )
            .await
        }
        Entity::State => {
            gateway::create_sfn_integration(
                client,
                api_id,
                &int_name,
                role_arn,
                *is_async,
                request_params.clone(),
            )
            .await
        }
        Entity::Event => {
            gateway::create_event_integration(
                client,
                api_id,
                &int_name,
                role_arn,
                request_params.clone(),
            )
            .await
        }
        Entity::Queue => {
            gateway::create_sqs_integration(
                client,
                api_id,
                &int_name,
                role_arn,
                request_params.clone(),
            )
            .await
        }
        _ => todo!(),
    }
}

fn resolve_route(gateway: &Gateway, route: &Route) -> Route {
    match std::env::var("TC_DISABLE_ROUTE_TRIM") {
        Ok(_) => route.clone(),
        Err(_) => {
            let mut r: Route = route.clone();
            for path in &gateway.paths {
                let abs_path = format!("/{}", path);
                if route.path.starts_with(&abs_path) {
                    let path = r.path.replace(&abs_path, "");
                    r.path = path;
                    return r;
                } else {
                    return r;
                }
            }
            return r;
        }
    }
}

async fn create_route(
    auth: &Auth,
    route: &Route,
    api_id: &str,
    auth_id: Option<String>,
    auth_kind: &str,
) {
    add_target_permission(auth, &api_id, &route.target).await;
    let client = gateway::make_client(auth).await;
    let integration_id = create_integration(auth, &client, &api_id, route, &route.target).await;
    gateway::create_or_update_route(
        &client,
        &api_id,
        &route.method,
        &route.path,
        &integration_id,
        auth_id,
        auth_kind,
    )
    .await;
}

async fn create_cognito_pool(auth: &Auth, pool_name: &str) -> (String, String) {
    let client = cognito::make_client(auth).await;

    let (id, client_id) = cognito::create_or_update_auth_pool(&client, pool_name).await;
    let issuer = format!("https://cognito-idp.{}.amazonaws.com/{}", &auth.region, &id);
    (issuer, client_id)
}

async fn add_auth_permission(auth: &Auth, lambda_arn: &str, api_id: &str, auth_name: &str) {
    let client = lambda::make_client(auth).await;
    let source_arn = auth.authorizer_arn(api_id, auth_name);
    let principal = "apigateway.amazonaws.com";
    let _ = lambda::add_permission(client, lambda_arn, principal, &source_arn, api_id).await;
}

async fn create_authorizer(
    auth: &Auth,
    api_id: &str,
    maybe_authorizer: Option<Authorizer>,
) -> (Option<String>, String) {
    if let Some(authorizer) = maybe_authorizer {
        let client = gateway::make_client(auth).await;
        if authorizer.create {
            match authorizer.kind.as_ref() {
                "lambda" => {
                    let uri = auth.lambda_uri(&authorizer.name);
                    let lambda_arn = auth.lambda_arn(&authorizer.name);

                    add_auth_permission(auth, &lambda_arn, &api_id, &authorizer.name).await;
                    let id = gateway::create_or_update_lambda_authorizer(
                        &client,
                        &api_id,
                        &authorizer.name,
                        &uri,
                    )
                    .await;
                    (Some(id), authorizer.kind)
                }
                "cognito" => {
                    let (issuer, client_id) = create_cognito_pool(auth, &authorizer.name).await;
                    let id = gateway::create_or_update_cognito_authorizer(
                        &client,
                        &api_id,
                        &authorizer.name,
                        &issuer,
                        &client_id,
                    )
                    .await;
                    (Some(id), authorizer.kind)
                }
                _ => (None, String::from("")),
            }
        } else {
            (None, String::from(""))
        }
    } else {
        (None, String::from(""))
    }
}

fn find_throttling(
    throttling: &HashMap<String, HashMap<String, Throttling>>,
    env: &str,
    sandbox: &str,
) -> (Option<i32>, Option<f64>) {
    match throttling.get(env) {
        Some(e) => {
            let maybe_t = e.get(sandbox);
            if let Some(t) = maybe_t {
                (t.burst_limit, t.rate_limit)
            } else {
                (None, None)
            }
        }
        None => match throttling.get("default") {
            Some(d) => {
                let maybe_t = d.get(sandbox);
                if let Some(t) = maybe_t {
                    (t.burst_limit, t.rate_limit)
                } else {
                    (None, None)
                }
            }
            None => (None, None),
        },
    }
}

async fn update_dns_record(auth: &Auth, domain: &str, cname: &str, target_zone_id: Option<String>) {
    tracing::debug!("Associating domain {}", domain);
    let rclient = route53::make_client(auth).await;
    route53::create_record_set(&rclient, domain, "CNAME", cname, target_zone_id).await;
}

async fn find_or_create_cert(auth: &Auth, domain: &str, token: &str) -> String {
    let client = acm::make_region_client(auth).await;

    let maybe_cert = acm::find_cert(&client, domain).await;
    let cert_arn = if let Some(arn) = maybe_cert {
        tracing::debug!("Cert already exists {}", &arn);
        arn
    } else {
        println!("Creating cert {}", domain);
        acm::request_cert(&client, domain, token).await
    };
    u::sleep(1000);
    if !acm::is_cert_issued(&client, &cert_arn).await {
        u::sleep(10000);
        let validation_records = acm::get_domain_validation_records(&client, &cert_arn).await;
        let route53_client = route53::make_client(auth).await;
        for rec in validation_records {
            route53::create_validation_record_set(
                &route53_client,
                domain,
                &rec.name,
                &rec.r#type.as_str(),
                &rec.value,
            )
            .await;
        }
        acm::wait_until_validated(&client, &cert_arn).await;
    } else {
        println!("Checking cert status: Issued");
    }
    cert_arn
}

fn make_cors(route: &Route) -> Option<Cors> {
    let mut methods: Vec<String> = vec![];
    let mut origins: Vec<String> = vec![];
    let mut headers: Vec<String> = vec![];
    if let Some(c) = &route.cors {
        methods.extend(c.methods.clone());
        origins.extend(c.origins.clone());
        headers.extend(c.headers.clone());
    }
    if origins.is_empty() {
        None
    } else {
        Some(gateway::make_cors(
            methods.into_iter().unique().collect(),
            origins.into_iter().unique().collect(),
            Some(headers.into_iter().unique().collect()),
        ))
    }
}

#[derive(Clone, Debug)]
struct Gateway {
    stage: String,
    cors: Option<Cors>,
    authorizer: Option<Authorizer>,
    burst_limit: Option<i32>,
    rate_limit: Option<f64>,
    domain: Option<String>,
    paths: Vec<String>,
    manage: bool,
}

fn collate_gateways(
    routes: &HashMap<String, Route>,
    env: &str,
    sandbox: &str,
) -> HashMap<String, Gateway> {
    let default_route = routes.get("default");

    let mut h: HashMap<String, Gateway> = HashMap::new();

    if let Some(route) = default_route {
        let gw = Gateway {
            stage: route.stage.clone(),
            cors: make_cors(&route),
            authorizer: route.authorizer.clone(),
            burst_limit: None,
            rate_limit: None,
            domain: None,
            paths: vec![],
            manage: false,
        };
        h.insert(route.gateway.clone(), gw);
    }
    for (name, route) in routes {
        if !&route.skip && name != "default" {
            let Route {
                gateway,
                stage,
                domains,
                authorizer,
                throttling,
                verticals,
                ..
            } = route;

            // throttling
            let (burst_limit, rate_limit) = find_throttling(&throttling, env, sandbox);

            // domains
            let maybe_domain = match domains.get(env) {
                Some(e) => e.get(sandbox).cloned(),
                None => match domains.get("default") {
                    Some(d) => d.get(sandbox).cloned(),
                    None => None,
                },
            };

            // gateway mapping paths
            let paths = if let Some(domain) = maybe_domain.clone() {
                match verticals.get(&domain) {
                    Some(m) => match m.get(gateway) {
                        Some(paths) => {
                            let mut xs: Vec<String> = vec![];
                            for p in paths {
                                if p.starts_with("/") {
                                    xs.push(p[1..].to_string())
                                } else {
                                    xs.push(p.clone())
                                };
                            }
                            xs
                        }
                        None => vec![],
                    },
                    None => vec![],
                }
            } else {
                vec![]
            };

            let manage = gateway.ends_with(&format!("_{}", sandbox));

            if !gateway.is_empty() {
                let gw = Gateway {
                    cors: make_cors(&route),
                    authorizer: authorizer.clone(),
                    stage: stage.to_string(),
                    burst_limit: burst_limit,
                    rate_limit: rate_limit,
                    domain: maybe_domain,
                    paths: paths,
                    manage: manage,
                };
                h.insert(gateway.to_string(), gw);
            }
        }
    }
    h
}

#[derive(Clone, Debug)]
struct GatewayState {
    api_id: String,
    auth_id: Option<String>,
    auth_kind: String,
    stage: String,
    endpoint: String,
}

async fn create_or_update_gateways(
    auth: &Auth,
    gateways: HashMap<String, Gateway>,
    tags: &HashMap<String, String>,
    sandbox: &str,
) -> HashMap<String, GatewayState> {
    let mut h: HashMap<String, GatewayState> = HashMap::new();
    let client = gateway::make_client(auth).await;

    for (name, gateway) in gateways {
        let Gateway {
            cors,
            authorizer,
            stage,
            burst_limit,
            rate_limit,
            domain,
            paths,
            manage,
            ..
        } = gateway;
        println!("Creating route: gateway {} manage: {}", name, manage);
        if !manage {
            let maybe_api_id = gateway::find_api(&client, &name).await;
            if let Some(api_id) = maybe_api_id {
                let (auth_id, auth_kind) = create_authorizer(auth, &api_id, authorizer).await;

                let gs = GatewayState {
                    api_id: api_id,
                    auth_id: auth_id,
                    auth_kind: auth_kind,
                    stage: stage,
                    endpoint: String::from(""),
                };
                h.insert(name, gs);
            } else {
                println!("Gateway {} not managed, ignoring.. ", name);
            }
        } else {
            let api_id =
                gateway::create_or_update_api(&client, &name, cors.clone(), tags.clone()).await;
            if cors.is_none() {
                gateway::clear_cors(&client, &api_id).await;
            }
            let (auth_id, auth_kind) = create_authorizer(auth, &api_id, authorizer).await;

            gateway::create_or_update_stage(&client, &api_id, &stage, burst_limit, rate_limit)
                .await;

            let endpoint = if let Some(dom) = domain {
                println!("Creating domain: {}", dom);
                let idempotency_token = sandbox;
                let cert_arn = find_or_create_cert(auth, &dom, idempotency_token).await;
                let client = gateway::make_client(auth).await;
                let gateway_domain = gateway::create_or_update_domain(
                    &client, &api_id, &dom, &stage, &cert_arn, paths,
                )
                .await;
                let target_zone_id = gateway::find_hosted_zone(&client, &dom).await;
                println!("Updating dns record {}", &gateway_domain);
                update_dns_record(auth, &dom, &gateway_domain, target_zone_id).await;
                dom
            } else {
                auth.api_endpoint(&api_id, &stage)
            };

            let gs = GatewayState {
                api_id: api_id,
                auth_id: auth_id,
                auth_kind: auth_kind,
                stage: stage,
                endpoint: endpoint,
            };
            h.insert(name, gs);
        }
    }
    h
}

pub async fn create(
    auth: &Auth,
    routes: &HashMap<String, Route>,
    tags: &HashMap<String, String>,
    sandbox: &str,
) {
    if routes.len() > 0 {
        let gateways = collate_gateways(routes, &auth.name, sandbox);
        let client = gateway::make_client(auth).await;

        let gateway_states =
            create_or_update_gateways(&auth, gateways.clone(), tags, sandbox).await;

        let mut url: String = String::from("");

        for (name, route) in routes {
            if !&route.skip && name != "default" {
                if let Some(gs) = gateway_states.get(&route.gateway) {
                    let gw = gateways.get(&route.gateway).unwrap();

                    let GatewayState {
                        api_id,
                        auth_id,
                        auth_kind,
                        endpoint,
                        stage,
                    } = gs;

                    tracing::debug!("Creating route {} {}", &route.method, &route.path);
                    let res_route = resolve_route(&gw, &route);
                    match route.authorizer {
                        Some(_) => {
                            create_route(auth, &res_route, &api_id, auth_id.clone(), &auth_kind)
                                .await
                        }
                        None => create_route(auth, &res_route, &api_id, None, &auth_kind).await,
                    };

                    let gateway_arn = auth.api_gateway_arn(&api_id);

                    gateway::create_deployment(&client, &api_id, &stage).await;
                    gateway::update_tags(&client, &gateway_arn, tags.clone()).await;
                    url = endpoint.to_string();
                }
            } else {
                println!("Skipping routes");
            }
        }
        println!("Endpoint {}", url);
    }
}

async fn delete_integration(client: &Client, api_id: &str, method: &str, target: &Target) {
    let Target { entity, arn, .. } = target;
    let int_name = format!("{}-{}", entity.to_str(), method);
    match entity {
        Entity::Function => gateway::delete_lambda_integration(client, api_id, arn).await,
        Entity::State => gateway::delete_sfn_integration(client, api_id, &int_name).await,
        Entity::Event => gateway::delete_event_integration(client, api_id, &int_name).await,
        Entity::Queue => gateway::delete_sqs_integration(client, api_id, &int_name).await,
        _ => (),
    }
}

async fn delete_route(client: &Client, api_id: &str, route: &Route) {
    let route_key = format!("{} {}", &route.method, &route.path);
    let route_id = gateway::find_route(client, api_id, &route_key).await;
    match route_id {
        Some(rid) => {
            println!("Deleting route: {}", &route_key);
            gateway::delete_route(client, &api_id, &rid).await.unwrap();
        }
        _ => (),
    }
    delete_integration(client, &api_id, &route.method, &route.target).await;
}

pub async fn delete(auth: &Auth, routes: &HashMap<String, Route>, sandbox: &str, force: bool) {
    if routes.len() > 0 {
        let client = gateway::make_client(auth).await;

        let gateways = collate_gateways(routes, &auth.name, sandbox);

        for (name, gateway) in gateways {
            let maybe_api_id = gateway::find_api(&client, &name).await;
            if let Some(api_id) = maybe_api_id {
                for (name, route) in routes {
                    println!("Deleting route {}", &name);
                    if !&route.skip {
                        delete_route(&client, &api_id, &route).await;
                    }
                }
                if gateway.manage {
                    if let Some(authorizer) = gateway.authorizer {
                        gateway::delete_authorizer(&client, &api_id, &authorizer.name).await;
                    }
                    if let Some(domain) = gateway.domain {
                        println!("Deleting api mappings {}", &api_id);
                        gateway::delete_api_mappings(&client, &api_id, &domain, gateway.paths)
                            .await;
                    }
                    if force {
                        gateway::delete_api(&client, &api_id).await;
                    }
                }
            }
        }
    }
}

pub async fn update(_auth: &Auth, _mutations: &HashMap<String, Route>, _c: &str) {}

pub async fn create_dry_run(routes: &HashMap<String, Route>) {
    for (_, route) in routes {
        println!("Creating route {} {}", &route.method, &route.path);
    }
}

pub async fn config(auth: &Auth, name: &str) -> HashMap<String, String> {
    let client = gateway::make_client(auth).await;
    let maybe_api_id = gateway::find_api_id(&client, name).await;
    match maybe_api_id {
        Some(api_id) => {
            let mut h: HashMap<String, String> = HashMap::new();
            let endpoint = auth.api_endpoint(&api_id, "$default");
            h.insert(s!("REST_ENDPOINT"), endpoint);

            let cognito_client = cognito::make_client(auth).await;
            let (maybe_pool_id, maybe_client_id) = cognito::get_config(&cognito_client, name).await;
            if let Some(pool_id) = maybe_pool_id {
                let issuer = format!(
                    "https://cognito-idp.{}.amazonaws.com/{}",
                    &auth.region, &pool_id
                );
                h.insert(s!("OIDC_AUTHORITY"), issuer);
            }
            if let Some(client_id) = maybe_client_id {
                h.insert(s!("OIDC_CLIENT_ID"), client_id);
            }
            h
        }
        _ => HashMap::new(),
    }
}

pub async fn freeze(auth: &Auth, fqn: &str) {
    let client = gateway::make_client(auth).await;
    let maybe_api_id = gateway::find_api(&client, fqn).await;
    if let Some(api_id) = maybe_api_id {
        let arn = auth.api_gateway_arn(&api_id);
        let version = gateway::get_tag(&client, &arn, s!("version")).await;
        if &version != "0.0.1" && !&version.is_empty() {
            println!("Freezing routes {} ({})", fqn, version);
            let kv = u::kv("freeze", "true");
            let _ = gateway::update_tags(&client, &arn, kv).await;
        }
    }
}

pub async fn unfreeze(auth: &Auth, fqn: &str) {
    let client = gateway::make_client(auth).await;
    let maybe_api_id = gateway::find_api(&client, fqn).await;
    if let Some(api_id) = maybe_api_id {
        let arn = auth.api_gateway_arn(&api_id);
        let version = gateway::get_tag(&client, &arn, s!("version")).await;
        if &version != "0.0.1" && !&version.is_empty() {
            println!("Unfreezing routes {} ({})", fqn, version);
            let kv = u::kv("freeze", "false");
            let _ = gateway::update_tags(&client, &arn, kv).await;
        }
    }
}

pub async fn is_frozen(auth: &Auth, fqn: &str) -> bool {
    let client = gateway::make_client(auth).await;
    let maybe_api_id = gateway::find_api(&client, fqn).await;

    if let Some(api_id) = maybe_api_id {
        let arn = auth.api_gateway_arn(&api_id);
        let v = gateway::get_tag(&client, &arn, s!("freeze")).await;
        v == "true"
    } else {
        false
    }
}
