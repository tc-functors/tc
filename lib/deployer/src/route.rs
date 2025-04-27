use compiler::{
    Route,
    Entity
};
use log::info;
use provider::{
    Env,
    aws::{
        gateway,
        gateway::Api,
        lambda,
    },
};
use std::collections::HashMap;

async fn make_api(env: &Env, role: &str, route: &Route) -> Api {
    let client = gateway::make_client(env).await;
    let uri = env.sfn_uri();

    Api {
        name: route.to_owned().gateway,
        client: client,
        stage: route.stage.to_owned(),
        stage_variables: route.stage_variables.to_owned(),
        uri: uri,
        role: role.to_string(),
        path: route.to_owned().path,
        authorizer: route.to_owned().authorizer,
        method: route.method.to_owned(),
        request_template: route.request_template.clone(),
        response_template: route.response_template.clone(),
    }
}

async fn add_permission(env: &Env, lambda_arn: &str, api_id: &str) {
    let client = lambda::make_client(env).await;
    let source_arn = env.api_arn(api_id);
    let principal = "apigateway.amazonaws.com";
    let _ = lambda::add_permission(
        client, lambda_arn, principal, &source_arn, api_id
    ).await;
}

async fn create_api(
    env: &Env,
    api: &Api,
    integration_type: &Entity,
    target_arn: &str
) {

    let api_id = api.find_or_create().await;

    add_permission(env, target_arn, &api_id).await;
    let arn = env.api_integration_arn(target_arn);

    let integration_id = match integration_type {
        Entity::Function => {
            api.create_integration(&api_id, "lambda", &arn).await
        }
        Entity::State => {
            api.create_integration(&api_id, "sfn", &arn).await
        }
        _ => String::from("")

    };

    let authorizer_id = api.find_authorizer(&api_id).await;
    api.find_or_create_route(&api_id, &integration_id, authorizer_id).await;
    api.create_stage(&api_id).await;
    api.create_deployment(&api_id, &api.stage).await;

    let endpoint = env.api_endpoint(&api_id, &api.stage);
    println!("Endpoint {}", &endpoint);
}

async fn create_route(env: &Env, route: &Route, role: &str) {
    let api = make_api(env, role, route).await;
    create_api(env, &api, &route.target_kind, &route.target_arn).await;
}

pub async fn create(env: &Env, role: &str, routes: HashMap<String, Route>) {
    for (_, route) in routes {
        println!("Creating route {} {}", &route.method, &route.path);
        create_route(env, &route, role).await;
    }
}

async fn delete_route(env: &Env, route: &Route, role: &str) {
    let api = make_api(env, role, route).await;
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
            //api.delete(&id).await
        }
        _ => (),
    }

}

pub async fn delete(env: &Env, role: &str, routes: HashMap<String, Route>) {
    for (name, route) in routes {
        info!("Deleting route {}", &name);
        delete_route(env, &route, role).await;

    }
}
