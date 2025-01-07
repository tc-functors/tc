use aws::gateway;
use aws::gateway::RestApi;
use aws::gatewayv2;
use aws::gatewayv2::Api;
use aws::lambda;
use std::collections::HashMap;

use crate::resolver::Route;
use aws::Env;
use kit as u;
use kit::*;
use log::info;

fn request_template_rest(sfn_arn: &str) -> String {
    format!(
        r#"#set($input = $input.json('$'))
         {{
           "input": "$util.escapeJavaScript($input)",
           "stateMachineArn": "{sfn_arn}"
         }}"#
    )
}

fn request_template_http() -> String {
    s!("\"{\"path\": $request.path, \"body\": $request.body}\"")
}

fn response_template() -> String {
    format!(r#"#set ($parsedPayload = $util.parseJson($input.json('$.output'))) $parsedPayload"#)
}

async fn make_rest_api(env: &Env, sfn_arn: &str, role: &str, route: &Route) -> RestApi {
    let client = gateway::make_client(env).await;
    let uri = env.sfn_uri();

    let path = u::second(&route.path, "/");

    let req_templ = request_template_rest(sfn_arn);
    let res_templ = response_template();

    RestApi {
        name: route.to_owned().gateway,
        client: client,
        stage: String::from("test"),
        uri: uri,
        role: role.to_string(),
        path: path,
        method: route.to_owned().method,
        request_template: req_templ,
        response_template: res_templ,
    }
}

async fn make_api(env: &Env, role: &str, route: &Route) -> Api {
    let client = gatewayv2::make_client(env).await;
    let uri = env.sfn_uri();

    let req_templ = request_template_http();
    let res_templ = response_template();

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
        request_template: req_templ,
        response_template: res_templ,
    }
}

async fn create_rest_api(env: &Env, api: &RestApi) {
    let name = &api.name;

    let api_id = api.find_or_create().await;
    info!("Created API {} id: {}", &name, &api_id);

    let root_id = api.get_root_resource(&api_id).await;

    info!(
        "Creating resource on root: {} path: {}",
        &root_id, &api.path
    );
    let resource_id = api
        .find_or_create_resource(&api_id, &root_id, &api.path)
        .await;

    info!("Adding method {} to resource {}", &api.method, &resource_id);
    let _ = api.put_method(&api_id, &resource_id, &api.method).await;

    info!("Adding Integration to resource {}", &resource_id);
    let _ = api
        .put_integration_request(&api_id, &resource_id, &api.method)
        .await;
    let _ = api
        .put_integration_response(&api_id, &resource_id, &api.method)
        .await;
    let _ = api
        .put_method_response(&api_id, &resource_id, &api.method)
        .await;

    let _ = api.create_deployment(&api_id, &api.stage).await;

    if kit::trace() {
        let endpoint = env.api_endpoint(&api_id, &api.stage);
        println!(
            "curl -X {} {}{} -d @payload.json -H \"Content-Type: application/json\"",
            &api.method, &endpoint, &api.path
        );
    }
}

async fn add_permission(env: &Env, lambda_arn: &str, api_id: &str) {
    let client = lambda::make_client(env).await;
    let source_arn = env.api_arn(api_id);
    let principal = "apigateway.amazonaws.com";
    let _ = lambda::add_permission(client, lambda_arn, principal, &source_arn, api_id).await;
}

async fn create_api(env: &Env, api: &Api, integration_type: &str, lambda_arn: &str) {
    let api_id = api.find_or_create().await;

    add_permission(env, lambda_arn, &api_id).await;
    let arn = env.api_integration_arn(lambda_arn);

    let integration_id = match integration_type {
        "lambda" => api.find_or_create_lambda_integration(&api_id, &arn).await,
        "sfn" => api.find_or_create_sfn_integration(&api_id, &arn).await,
        _ => panic!("No integration specified"),
    };

    let authorizer_id = api.find_authorizer(&api_id).await;
    let _ = api
        .find_or_create_route(&api_id, &integration_id, authorizer_id)
        .await;

    let _ = api.create_stage(&api_id).await.unwrap();
    let _ = api.create_deployment(&api_id, &api.stage).await;

    if kit::trace() {
        let endpoint = env.api_endpoint(&api_id, &api.stage);
        println!(
            "curl -X {} {}{} -d @payload.json -H \"Content-Type: application/json\"",
            &api.method, &endpoint, &api.path
        );
    }
}

async fn find_or_create_proxy(env: &Env, proxy: &str) -> Option<String> {
    match proxy {
        "default" | "tc-sfn-router" | "sfn-router" => Some(env.lambda_arn("sfn-router")),
        "none" => None,
        _ => Some(env.lambda_arn(proxy)),
    }
}

async fn create_route(env: &Env, route: &Route, sfn_arn: &str, role: &str) {
    match route.kind.as_str() {
        "rest" => {
            let rest_api = make_rest_api(env, sfn_arn, role, route).await;
            create_rest_api(env, &rest_api).await;
        }
        "http" => {
            let api = make_api(env, role, route).await;
            let lambda_arn = find_or_create_proxy(env, &route.proxy).await;
            match lambda_arn {
                Some(arn) => {
                    create_api(env, &api, "lambda", &arn).await;
                }
                _ => create_api(env, &api, "sfn", sfn_arn).await,
            }
        }
        _ => (),
    }
}

pub async fn create(env: &Env, sfn_arn: &str, role: &str, routes: HashMap<String, Route>) {
    for (_, route) in routes {
        create_route(env, &route, sfn_arn, role).await;
    }
}

async fn delete_route(env: &Env, route: &Route, sfn_arn: &str, role: &str) {
    match route.kind.as_str() {
        "rest" => {
            let api = make_rest_api(env, sfn_arn, role, route).await;
            let api_id = api.clone().find().await;
            match api_id {
                Some(id) => api.clone().delete(&id).await,
                _ => (),
            }
        }

        "http" => {
            let api = make_api(env, role, route).await;
            let api_id = api.clone().find().await;
            let route_key = format!("{} {}", &route.method, &route.path);

            match api_id {
                Some(id) => {
                    let route_id = api.find_route(&id, &route_key).await;
                    match route_id {
                        Some(rid) => api.delete_route(&id, &rid).await.unwrap(),
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        _ => (),
    }
}

pub async fn delete(env: &Env, sfn_arn: &str, role: &str, routes: HashMap<String, Route>) {
    for (name, route) in routes {
        info!("Deleting route {}", &name);
        delete_route(env, &route, sfn_arn, role).await;
    }
}
