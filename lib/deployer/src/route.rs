use crate::aws::{
    gateway,
    gateway::Api,
    lambda,
};
use authorizer::Auth;
use aws_sdk_apigatewayv2::types::Cors;
use composer::{
    Entity,
    Route,
};
use itertools::Itertools;
use kit::*;
use std::collections::HashMap;

fn make_cors(routes: &HashMap<String, Route>) -> Option<Cors> {
    let mut methods: Vec<String> = vec![];
    let mut origins: Vec<String> = vec![];
    let mut headers: Vec<String> = vec![];
    for (_, route) in routes {
        if let Some(c) = &route.cors {
            methods.extend(c.methods.clone());
            origins.extend(c.origins.clone());
            if let Some(h) = &c.headers {
                headers.extend(h.clone());
            }
        }
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

async fn make_api(auth: &Auth, route: &Route, cors: Option<Cors>, tags: &HashMap<String, String>) -> Api {
    let client = gateway::make_client(auth).await;

    Api {
        name: route.to_owned().gateway,
        client: client,
        stage: route.stage.to_owned(),
        stage_variables: route.stage_variables.to_owned(),
        role: route.role_arn.to_string(),
        path: route.to_owned().path,
        method: route.method.to_owned(),
        sync: route.sync.to_owned(),
        request_template: route.request_template.clone(),
        cors: cors,
        tags: tags.clone()
    }
}

async fn add_permission(auth: &Auth, lambda_arn: &str, api_id: &str) {
    let client = lambda::make_client(auth).await;
    let source_arn = auth.api_arn(api_id);
    let principal = "apigateway.amazonaws.com";
    let _ = lambda::add_permission(client, lambda_arn, principal, &source_arn, api_id).await;
}

async fn add_auth_permission(auth: &Auth, lambda_arn: &str, api_id: &str, auth_name: &str) {
    let client = lambda::make_client(auth).await;
    let source_arn = auth.authorizer_arn(api_id, auth_name);
    let principal = "apigateway.amazonaws.com";
    let _ = lambda::add_permission(client, lambda_arn, principal, &source_arn, api_id).await;
}

fn integration_name(entity: &Entity, api: &Api) -> String {
    format!("{}-{}", entity.to_str(), api.method)
}

fn make_request_params(entity: &Entity, api: &Api, target_arn: &str) -> HashMap<String, String> {
    let mut req: HashMap<String, String> = HashMap::new();

    let name = integration_name(entity, api);

    // TODO: get target for event and queue

    match entity {
        Entity::State => {
            req.insert(s!("StateMachineArn"), s!(target_arn));
            req.insert(s!("Name"), name);
            req.insert(s!("Input"), api.request_template.to_string());
        }
        Entity::Event => {
            req.insert(s!("Detail"), s!(""));
            req.insert(s!("DetailType"), s!(""));
            req.insert(s!("Source"), s!(""));
        }
        Entity::Queue => {
            req.insert(s!("QueueUrl"), s!(""));
            req.insert(s!("MessageBody"), s!(""));
        }
        _ => (),
    }
    req
}

async fn create_integration(entity: &Entity, api: &Api, api_id: &str, target_arn: &str) -> String {
    let req_params = make_request_params(entity, api, target_arn);
    let int_name = integration_name(entity, api);
    match entity {
        Entity::Function => api.create_lambda_integration(api_id, target_arn).await,
        Entity::State => {
            api.create_sfn_integration(api_id, &int_name, req_params)
                .await
        }
        Entity::Event => {
            api.create_event_integration(api_id, &int_name, req_params)
                .await
        }
        Entity::Queue => {
            api.create_sqs_integration(api_id, &int_name, req_params)
                .await
        }
        _ => todo!(),
    }
}

async fn create_authorizer(
    auth: &Auth,
    api: &Api,
    api_id: &str,
    authorizer: &str,
) -> Option<String> {
    let uri = auth.lambda_uri(authorizer);
    let lambda_arn = auth.lambda_arn(authorizer);
    if authorizer.is_empty() {
        None
    } else {
        add_auth_permission(auth, &lambda_arn, &api_id, authorizer).await;
        let authorizer_id = api
            .create_or_update_authorizer(&api_id, authorizer, &uri)
            .await;
        Some(authorizer_id)
    }
}

async fn create_api(
    auth: &Auth,
    api: &Api,
    api_id: &str,
    entity: &Entity,
    target_arn: &str,
    auth_id: Option<String>,
) {
    add_permission(auth, target_arn, &api_id).await;

    let integration_id = create_integration(entity, api, &api_id, target_arn).await;

    api.find_or_create_route(&api_id, &integration_id, auth_id)
        .await;
    api.create_stage(&api_id).await;
    api.create_deployment(&api_id, &api.stage).await;

    let endpoint = auth.api_endpoint(&api_id, &api.stage);
    println!("Endpoint {}", &endpoint);
}

async fn create_route(auth: &Auth, route: &Route, cors: Option<Cors>, tags: &HashMap<String, String>) {
    let api = make_api(auth, route, cors, tags).await;
    let api_id = api.create_or_update().await;
    let auth_id = if route.create_authorizer {
        match &route.authorizer {
            Some(authorizer) => create_authorizer(auth, &api, &api_id, &authorizer).await,
            None => None,
        }
    } else {
        None
    };
    create_api(
        auth,
        &api,
        &api_id,
        &route.entity,
        &route.target_arn,
        auth_id,
    )
    .await;
}

pub async fn create(auth: &Auth, routes: &HashMap<String, Route>, tags: &HashMap<String, String>) {
    let cors = make_cors(&routes);
    tracing::debug!("Updating cors: {:?}", cors);
    for (_, route) in routes {
        tracing::debug!("Creating route {} {}", &route.method, &route.path);
        if !&route.skip {
            create_route(auth, &route, cors.clone(), tags).await;
        }
    }
}

async fn delete_integration(entity: &Entity, api: &Api, api_id: &str, target_arn: &str) {
    let int_name = integration_name(entity, api);
    match entity {
        Entity::Function => api.delete_lambda_integration(api_id, target_arn).await,
        Entity::State => api.delete_sfn_integration(api_id, &int_name).await,
        Entity::Event => api.delete_event_integration(api_id, &int_name).await,
        Entity::Queue => api.delete_sqs_integration(api_id, &int_name).await,
        _ => (),
    }
}

async fn delete_route(auth: &Auth, route: &Route) {
    let api = make_api(auth, route, None, &HashMap::new()).await;
    let api_id = api.clone().find().await;
    let route_key = format!("{} {}", &route.method, &route.path);

    match api_id {
        Some(id) => {
            let route_id = api.find_route(&id, &route_key).await;
            match route_id {
                Some(rid) => {
                    println!("Deleting route: {}", &route_key);
                    api.delete_route(&id, &rid).await.unwrap();
                }
                _ => (),
            }
            delete_integration(&route.entity, &api, &id, &route.target_arn).await;
            if route.create_authorizer {
                if let Some(authorizer) = &route.authorizer {
                    api.delete_authorizer(&id, &authorizer).await;
                }
            }

            match std::env::var("TC_DELETE_ROOT") {
                Ok(_) => api.delete(&id).await,
                Err(_) => (),
            }
        }
        _ => (),
    }
}

pub async fn delete(auth: &Auth, routes: &HashMap<String, Route>) {
    for (name, route) in routes {
        println!("Deleting route {}", &name);
        if !&route.skip {
            delete_route(auth, &route).await;
        }
    }
}

pub async fn update(_auth: &Auth, _mutations: &HashMap<String, Route>, _c: &str) {}
