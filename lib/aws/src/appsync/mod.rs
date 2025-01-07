use super::Env;
use aws_sdk_appsync::types::builders::{
    AdditionalAuthenticationProviderBuilder, LambdaAuthorizerConfigBuilder,
};
use aws_sdk_appsync::types::LambdaAuthorizerConfig;
use aws_sdk_appsync::types::{AdditionalAuthenticationProvider, AuthenticationType};
use aws_sdk_appsync::types::{ResolverKind, TypeDefinitionFormat};
use aws_sdk_appsync::Client;
use colored::Colorize;
use kit::*;
use std::collections::HashMap;

mod dynamodb;
mod event;
mod http;
mod lambda;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

fn make_auth_type() -> AdditionalAuthenticationProvider {
    let auth_type = AuthenticationType::AwsIam;
    let v = AdditionalAuthenticationProviderBuilder::default();
    v.authentication_type(auth_type).build()
}

#[derive(Clone, Debug)]
pub struct Api {
    pub id: String,
    pub https: String,
    pub wss: String,
}

async fn list_apis(client: &Client) -> HashMap<String, Api> {
    let mut h: HashMap<String, Api> = HashMap::new();
    let r = client.list_graphql_apis().send().await;
    match r {
        Ok(res) => {
            let apis = res.graphql_apis.unwrap();
            for api in apis {
                let uris = api.uris.unwrap();
                let https = uris.get("GRAPHQL");
                let wss = uris.get("REALTIME");
                let a = Api {
                    id: api.api_id.unwrap().to_string(),
                    https: https.unwrap().to_string(),
                    wss: wss.unwrap().to_string(),
                };

                h.insert(api.name.unwrap(), a);
            }
        }
        Err(e) => panic!("{}", e),
    }
    h
}

pub async fn find_api(client: &Client, name: &str) -> Option<Api> {
    let apis = list_apis(client).await;
    apis.get(name).cloned()
}

fn make_lambda_authorizer(authorizer_arn: &str) -> LambdaAuthorizerConfig {
    let v = LambdaAuthorizerConfigBuilder::default();
    v.authorizer_uri(authorizer_arn).build().unwrap()
}

async fn create_api(
    client: &Client,
    name: &str,
    authorizer_arn: &str,
) -> (String, HashMap<String, String>) {
    println!("Creating api {}", name.green());
    let auth_type = AuthenticationType::AwsLambda;
    let lambda_auth_config = make_lambda_authorizer(authorizer_arn);
    let additional_auth_type = make_auth_type();
    let r = client
        .create_graphql_api()
        .name(s!(name))
        .authentication_type(auth_type)
        .additional_authentication_providers(additional_auth_type)
        .lambda_authorizer_config(lambda_auth_config)
        .send()
        .await;
    match r {
        Ok(res) => {
            let resp = res.graphql_api.unwrap();
            (resp.api_id.unwrap(), resp.uris.unwrap())
        }
        Err(e) => panic!("{}", e),
    }
}

async fn update_api(
    client: &Client,
    name: &str,
    authorizer_arn: &str,
    api_id: &str,
) -> (String, HashMap<String, String>) {
    println!("Updating api {}", name.blue());
    let auth_type = AuthenticationType::AwsLambda;
    let lambda_auth_config = make_lambda_authorizer(authorizer_arn);
    let additional_auth_type = make_auth_type();
    let r = client
        .update_graphql_api()
        .name(s!(name))
        .api_id(s!(api_id))
        .authentication_type(auth_type)
        .additional_authentication_providers(additional_auth_type)
        .lambda_authorizer_config(lambda_auth_config)
        .send()
        .await;
    match r {
        Ok(res) => {
            let resp = res.graphql_api.unwrap();
            (resp.api_id.unwrap(), resp.uris.unwrap())
        }
        Err(e) => panic!("{}", e),
    }
}

pub async fn create_or_update_api(
    client: &Client,
    name: &str,
    authorizer_arn: &str,
) -> (String, HashMap<String, String>) {
    let api = find_api(client, name).await;
    match api {
        Some(a) => update_api(client, name, authorizer_arn, &a.id).await,
        None => create_api(client, name, authorizer_arn).await,
    }
}

// types

async fn list_types(client: &Client, api_id: &str) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    let r = client
        .list_types()
        .api_id(s!(api_id))
        .format(TypeDefinitionFormat::Sdl)
        .send()
        .await;
    match r {
        Ok(res) => {
            let types = res.types.unwrap();
            for t in types {
                v.push(t.name.unwrap());
            }
        }
        Err(e) => panic!("{}", e),
    }
    v
}

async fn has_type(client: &Client, api_id: &str, name: &str) -> bool {
    let types = list_types(client, api_id).await;
    types.contains(&s!(name))
}

async fn create_type(client: &Client, api_id: &str, type_name: &str, definition: &str) {
    println!("Creating type {}", type_name.green());
    let _ = client
        .create_type()
        .api_id(s!(api_id))
        .definition(s!(definition))
        .format(TypeDefinitionFormat::Sdl)
        .send()
        .await
        .unwrap();
}

async fn update_type(client: &Client, api_id: &str, type_name: &str, definition: &str) {
    println!("Updating type {}", type_name.blue());
    let _ = client
        .update_type()
        .type_name(s!(type_name))
        .api_id(s!(api_id))
        .definition(s!(definition))
        .format(TypeDefinitionFormat::Sdl)
        .send()
        .await
        .unwrap();
}

pub async fn create_or_update_type(
    client: &Client,
    api_id: &str,
    type_name: &str,
    definition: &str,
) {
    if has_type(client, api_id, type_name).await {
        update_type(client, api_id, type_name, definition).await
    } else {
        create_type(client, api_id, type_name, definition).await
    }
}

// datastore

async fn has_datasource(client: &Client, api_id: &str, name: &str) -> bool {
    let r = client
        .get_data_source()
        .api_id(s!(api_id))
        .name(s!(name))
        .send()
        .await;
    match r {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub struct DatasourceInput {
    pub kind: String,
    pub name: String,
    pub role_arn: String,
    pub target_arn: String,
    pub config: HashMap<String, String>,
}

pub async fn find_or_create_datasource(client: &Client, api_id: &str, datasource: DatasourceInput) {
    let DatasourceInput {
        kind,
        role_arn,
        name,
        target_arn,
        ..
    } = datasource;
    let exists = has_datasource(client, api_id, &name).await;
    match kind.as_ref() {
        "lambda" | "function" => {
            if exists {
                lambda::update_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            } else {
                lambda::create_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            }
        }
        "event" => {
            if exists {
                event::update_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            } else {
                event::create_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            }
        }
        "http" => {
            if exists {
                http::update_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            } else {
                http::create_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            }
        }
        "table" => {
            if exists {
                dynamodb::update_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            } else {
                dynamodb::create_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            }
        }
        _ => (),
    }
}

async fn list_functions(client: &Client, api_id: &str) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let r = client.list_functions().api_id(s!(api_id)).send().await;
    match r {
        Ok(res) => {
            let fns = res.functions.unwrap();
            for f in fns {
                h.insert(f.name.unwrap(), f.function_id.unwrap());
            }
        }
        Err(e) => panic!("{}", e),
    }
    h
}

async fn find_function(client: &Client, api_id: &str, name: &str) -> Option<String> {
    let fns = list_functions(client, api_id).await;
    fns.get(name).cloned()
}

async fn create_function(client: &Client, api_id: &str, name: &str, datasource_name: &str) {
    println!("Creating resolver {}", name.green());
    let _ = client
        .create_function()
        .api_id(s!(api_id))
        .name(s!(name))
        .data_source_name(s!(datasource_name))
        .function_version(s!("2018-05-29"))
        .send()
        .await
        .unwrap();
}

async fn update_function(
    client: &Client,
    api_id: &str,
    name: &str,
    function_id: &str,
    datasource_name: &str,
) {
    println!("Updating resolver {}", name.blue());
    let _ = client
        .update_function()
        .api_id(s!(api_id))
        .function_id(s!(function_id))
        .name(s!(name))
        .data_source_name(s!(datasource_name))
        .function_version(s!("2018-05-29"))
        .send()
        .await
        .unwrap();
}

pub async fn create_or_update_function(
    client: &Client,
    api_id: &str,
    name: &str,
    datasource: &str,
) {
    let function = find_function(client, api_id, name).await;
    match function {
        Some(function_id) => update_function(client, api_id, name, &function_id, datasource).await,
        None => create_function(client, api_id, name, datasource).await,
    }
}

async fn resolver_exists(client: &Client, api_id: &str, type_name: &str, field_name: &str) -> bool {
    let r = client
        .get_resolver()
        .api_id(s!(api_id))
        .type_name(s!(type_name))
        .field_name(s!(field_name))
        .send()
        .await;
    match r {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn create_resolver(
    client: &Client,
    api_id: &str,
    type_name: &str,
    field_name: &str,
    datasource: &str,
) {
    println!("Creating resolver for field {}", field_name.green());
    let _ = client
        .create_resolver()
        .api_id(api_id)
        .type_name(s!(type_name))
        .field_name(s!(field_name))
        .data_source_name(s!(datasource))
        .kind(ResolverKind::Unit)
        .send()
        .await
        .unwrap();
}

pub async fn find_or_create_resolver(
    client: &Client,
    api_id: &str,
    field_name: &str,
    datasource: &str,
) {
    let type_name = "Mutation";
    let exists = resolver_exists(client, api_id, type_name, field_name).await;
    if !exists {
        create_resolver(client, api_id, type_name, field_name, datasource).await;
    }
}

// deletes

pub async fn delete_api(client: &Client, api_name: &str) {
    println!("Deleting api {}", api_name.red());
    let api = find_api(client, api_name).await;
    match api {
        Some(ap) => {
            let _ = client.delete_graphql_api().api_id(ap.id).send().await;
        }
        None => (),
    }
}

// getters
pub async fn get_api_endpoint(client: &Client, api_id: &str) -> Option<String> {
    let r = client.get_graphql_api().api_id(s!(api_id)).send().await;
    match r {
        Ok(res) => {
            let uris = res.graphql_api.unwrap().uris.unwrap();
            uris.get("GRAPHQL").cloned()
        }
        Err(_) => None,
    }
}

pub async fn create_types(env: &Env, api_id: &str, types: HashMap<String, String>) {
    let client = make_client(env).await;
    for (t, def) in types {
        create_or_update_type(&client, &api_id, &t, &def).await;
    }
}
