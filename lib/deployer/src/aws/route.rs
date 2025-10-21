use compiler::Entity;
use composer::{
    Route,
    aws::route::{Target, Authorizer},
};
use itertools::Itertools;
use kit::*;
use provider::{
    Auth,
    aws::{
        gateway,
        gateway::{
            Client,
            GatewayCors as Cors,
        },
        lambda,
        cognito
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

async fn create_integration(client: &Client, api_id: &str, route: &Route, target: &Target) -> String {
    let Target {
        entity,
        arn,
        request_params,
        ..
    } = target;

    let Route { role_arn, method, is_async, .. } = route;

    let int_name = format!("{}-{}", entity.to_str(), method);

    match entity {
        Entity::Function => gateway::create_lambda_integration(client, api_id, arn, role_arn, *is_async).await,
        Entity::State => {
            gateway::create_sfn_integration(client, api_id, &int_name, role_arn, *is_async, request_params.clone())
                .await
        }
        Entity::Event => {
            gateway::create_event_integration(client, api_id, &int_name, role_arn, request_params.clone())
                .await
        }
        Entity::Queue => {
            gateway::create_sqs_integration(client, api_id, &int_name, role_arn, request_params.clone())
                .await
        }
        _ => todo!(),
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
    let integration_id = create_integration(&client, &api_id, route, &route.target).await;
    gateway::create_or_update_route(&client, &api_id, &route.method, &route.path, &integration_id, auth_id, auth_kind).await;
}

// api

struct Api {
    name: String,
    authorizer: Option<Authorizer>,
    stage: String,
    cors: Option<Cors>,

}

fn make_cors(routes: &HashMap<String, Route>) -> Option<Cors> {
    let mut methods: Vec<String> = vec![];
    let mut origins: Vec<String> = vec![];
    let mut headers: Vec<String> = vec![];
    for (_, route) in routes {
        let c = &route.cors;
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


fn find_authorizer(routes: &HashMap<String, Route>) -> Option<Authorizer> {
    for (_, route) in routes {
        if let Some(authorizer) = &route.authorizer {
            return Some(authorizer.clone())
        }
    }
    None
}

fn find_gateway(routes: &HashMap<String, Route>) -> String {
    for (_, route) in routes {
        if !route.gateway.is_empty() {
            return route.gateway.clone()
        }
    }
    panic!("No gateway found")
}
impl Api {
    fn new(routes: &HashMap<String, Route>) -> Api {
        let cors = make_cors(&routes);
        let maybe_authorizer = find_authorizer(&routes);
        let gateway = find_gateway(&routes);

        Api {
            name:  gateway,
            authorizer: maybe_authorizer,
            stage: String::from("$default"),
            cors: cors,
        }
    }
}

async fn create_cognito_pool(auth: &Auth, pool_name: &str) -> (String, String) {
    let client = cognito::make_client(auth).await;

    let (id, client_id) = cognito::create_or_update_auth_pool(&client, pool_name).await;
    let issuer = format!("https://cognito-idp.{}.amazonaws.com/{}",
                         &auth.region, &id);
    (issuer, client_id)
}

async fn add_auth_permission(auth: &Auth, lambda_arn: &str, api_id: &str, auth_name: &str) {
    let client = lambda::make_client(auth).await;
    let source_arn = auth.authorizer_arn(api_id, auth_name);
    let principal = "apigateway.amazonaws.com";
    let _ = lambda::add_permission(client, lambda_arn, principal, &source_arn, api_id).await;
}

async fn create_authorizer(auth: &Auth, api_id: &str, maybe_authorizer: Option<Authorizer>) -> (Option<String>, String) {
    if let Some(authorizer) = maybe_authorizer {
        let client = gateway::make_client(auth).await;
        match authorizer.kind.as_ref() {
            "lambda" => {
                let uri = auth.lambda_uri(&authorizer.name);
                let lambda_arn = auth.lambda_arn(&authorizer.name);

                add_auth_permission(auth, &lambda_arn, &api_id, &authorizer.name).await;
                let id = gateway::create_or_update_lambda_authorizer(&client, &api_id, &authorizer.name, &uri)
                    .await;
                (Some(id), authorizer.kind)
            },
            "cognito" => {
                let (issuer, client_id) = create_cognito_pool(auth, &authorizer.name).await;
                let id = gateway::create_or_update_cognito_authorizer(&client, &api_id, &authorizer.name, &issuer, &client_id)
                    .await;
                (Some(id), authorizer.kind)
            },
            _ => (None, String::from(""))
        }
    } else {
        (None, String::from(""))
    }
}

pub async fn create(auth: &Auth, routes: &HashMap<String, Route>, tags: &HashMap<String, String>) {
    let client = gateway::make_client(auth).await;
    let api = Api::new(routes);
    let api_id = gateway::create_or_update_api(&client, &api.name, api.cors, tags.clone()).await;
    let (auth_id, auth_kind) =  create_authorizer(auth, &api_id, api.authorizer).await;

    for (_, route) in routes {
        tracing::debug!("Creating route {} {}", &route.method, &route.path);
        if !&route.skip {
            create_route(auth, &route, &api_id, auth_id.clone(), &auth_kind).await;
        }
    }
    gateway::create_stage(&client, &api_id, &api.stage, HashMap::new()).await;
    gateway::create_deployment(&client, &api_id, &api.stage).await;
    let endpoint = auth.api_endpoint(&api_id, &api.stage);
    println!("Endpoint {}", &endpoint);
}

async fn delete_integration(client: &Client, api_id: &str, method: &str, target: &Target) {
    let Target { entity, arn, .. } = target;
    let int_name = format!("{}-{}", entity.to_str(), method);
    match entity {
        Entity::Function => {
            gateway::delete_lambda_integration(client, api_id, arn).await
        }
        Entity::State => {
            gateway::delete_sfn_integration(client, api_id, &int_name).await
        }
        Entity::Event => {
            gateway::delete_event_integration(client, api_id, &int_name).await
        }
        Entity::Queue => {
            gateway::delete_sqs_integration(client, api_id, &int_name).await
        }
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


pub async fn delete(auth: &Auth, routes: &HashMap<String, Route>) {
    let client = gateway::make_client(auth).await;
    let api = Api::new(routes);
    let maybe_api_id = gateway::find_api(&client, &api.name).await;

    if let Some(api_id) = maybe_api_id {
        for (name, route) in routes {
            println!("Deleting route {}", &name);
            if !&route.skip {
                delete_route(&client, &api_id, &route).await;
            }
        }
        if let Some(authorizer) = api.authorizer {
            gateway::delete_authorizer(&client, &api_id, &authorizer.name).await;
        }

        match std::env::var("TC_DELETE_ROOT") {
            Ok(_) => gateway::delete_api(&client, &api_id).await,
            Err(_) => (),
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
                let issuer = format!("https://cognito-idp.{}.amazonaws.com/{}",
                                     &auth.region, &pool_id);
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
