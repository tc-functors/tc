use compiler::{
    Route,
    Entity
};
use log::info;
use authorizer::Auth;
use crate::{
    aws::{
        gateway,
        gateway::Api,
        lambda,
    },
};
use std::collections::HashMap;

async fn make_api(auth: &Auth, role: &str, route: &Route) -> Api {
    let client = gateway::make_client(auth).await;

    Api {
        name: route.to_owned().gateway,
        client: client,
        stage: route.stage.to_owned(),
        stage_variables: route.stage_variables.to_owned(),
        role: role.to_string(),
        path: route.to_owned().path,
        authorizer: route.to_owned().authorizer,
        method: route.method.to_owned(),
        sync: route.sync.to_owned(),
        request_template: route.request_template.clone(),
    }
}

async fn add_permission(auth: &Auth, lambda_arn: &str, api_id: &str) {
    let client = lambda::make_client(auth).await;
    let source_arn = auth.api_arn(api_id);
    let principal = "apigateway.amazonaws.com";
    let _ = lambda::add_permission(
        client, lambda_arn, principal, &source_arn, api_id
    ).await;
}

async fn create_api(
    auth: &Auth,
    api: &Api,
    integration_type: &Entity,
    target_arn: &str
) {

    let api_id = api.find_or_create().await;

    add_permission(auth, target_arn, &api_id).await;

    let integration_id = api.create_integration(
        &api_id,
        &integration_type.to_str(),
        target_arn,
    ).await;

    let authorizer_id = api.find_authorizer(&api_id).await;
    api.find_or_create_route(&api_id, &integration_id, authorizer_id).await;
    api.create_stage(&api_id).await;
    api.create_deployment(&api_id, &api.stage).await;

    let endpoint = auth.api_endpoint(&api_id, &api.stage);
    println!("Endpoint {}", &endpoint);
}

async fn create_route(auth: &Auth, route: &Route, role: &str) {
    let api = make_api(auth, role, route).await;
    create_api(auth, &api, &route.entity, &route.target_arn).await;
}

pub async fn create(auth: &Auth, role: &str, routes: HashMap<String, Route>) {
    for (_, route) in routes {
        println!("Creating route {} {}", &route.method, &route.path);
        create_route(auth, &route, role).await;
    }
}

async fn delete_route(auth: &Auth, route: &Route, role: &str) {
    let api = make_api(auth, role, route).await;
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
            api.delete_integration(&id, &route.entity.to_str(), &route.target_arn).await;
            //api.delete(&id).await
        }
        _ => (),
    }

}

pub async fn delete(auth: &Auth, role: &str, routes: HashMap<String, Route>) {
    for (name, route) in routes {
        info!("Deleting route {}", &name);
        delete_route(auth, &route, role).await;

    }
}
