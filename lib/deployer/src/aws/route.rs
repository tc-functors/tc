use compiler::Entity;
use composer::Route;
use composer::aws::route::Target;
use itertools::Itertools;
use provider::{
    Auth,
    aws::{
        gateway,
        gateway::{
            Api,
            GatewayCors as Cors,
        },
        lambda,
    },
};
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

async fn make_api(
    auth: &Auth,
    route: &Route,
    cors: Option<Cors>,
    tags: &HashMap<String, String>,
) -> Api {
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
        cors: cors,
        tags: tags.clone(),
    }
}

async fn add_target_permission(auth: &Auth, api_id: &str, target: &Target) {
    let Target { entity, arn, .. } = target;
    match entity {
        Entity::Function => {
            let client = lambda::make_client(auth).await;
            let source_arn = auth.api_arn(api_id);
            let principal = "apigateway.amazonaws.com";
            let _ = lambda::add_permission(client, arn, principal, &source_arn, api_id).await;
        },
        _ => ()
    }


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

async fn create_integration(api: &Api, api_id: &str, target: &Target) -> String {
    let Target { entity, arn, request_params, .. } = target;

    let int_name = integration_name(entity, api);

    match entity {
        Entity::Function => api.create_lambda_integration(api_id, arn).await,
        Entity::State => {
            api.create_sfn_integration(api_id, &int_name, request_params.clone())
                .await
        }
        Entity::Event => {
            api.create_event_integration(api_id, &int_name, request_params.clone())
                .await
        }
        Entity::Queue => {
            api.create_sqs_integration(api_id, &int_name, request_params.clone())
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
    target: &Target,
    auth_id: Option<String>,
) {
    add_target_permission(auth, &api_id, target).await;
    let integration_id = create_integration(api, &api_id, target).await;

    api.find_or_create_route(&api_id, &integration_id, auth_id)
        .await;
    api.create_stage(&api_id).await;
    api.create_deployment(&api_id, &api.stage).await;

    let endpoint = auth.api_endpoint(&api_id, &api.stage);
    println!("Endpoint {}", &endpoint);
}

async fn create_route(
    auth: &Auth,
    route: &Route,
    cors: Option<Cors>,
    tags: &HashMap<String, String>,
) {
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
        &route.target,
        auth_id,
    )
    .await;
}

pub async fn create(auth: &Auth, routes: &HashMap<String, Route>, tags: &HashMap<String, String>) {
    let cors = make_cors(&routes);
    for (_, route) in routes {
        tracing::debug!("Creating route {} {}", &route.method, &route.path);
        if !&route.skip {
            create_route(auth, &route, cors.clone(), tags).await;
        }
    }
}

async fn delete_integration(api: &Api, api_id: &str, target: &Target) {
    let Target { entity, arn, .. } = target;
    let int_name = integration_name(entity, api);
    match entity {
        Entity::Function => api.delete_lambda_integration(api_id, arn).await,
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
            delete_integration(&api, &id, &route.target).await;
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
            h
        }
        _ => HashMap::new(),
    }
}
