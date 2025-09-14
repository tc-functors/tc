use crate::Auth;
use aws_sdk_appsync::{
    Client,
    Error,
    types::{
        AdditionalAuthenticationProvider,
        AuthenticationType,
        LambdaAuthorizerConfig,
        ResolverKind,
        TypeDefinitionFormat,
        builders::{
            AdditionalAuthenticationProviderBuilder,
            LambdaAuthorizerConfigBuilder,
        },
    },
};
use colored::Colorize;
use kit::*;
use std::collections::HashMap;
use tracing::debug;

mod dynamodb;
mod eventbridge;
pub mod events;
mod http;
mod lambda;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

fn make_auth_type() -> AdditionalAuthenticationProvider {
    let auth_type = AuthenticationType::AwsIam;
    let v = AdditionalAuthenticationProviderBuilder::default();
    v.authentication_type(auth_type).build()
}

async fn list_apis_by_token(
    client: &Client,
    token: &str,
) -> (HashMap<String, String>, Option<String>) {
    let res = client
        .list_graphql_apis()
        .next_token(token.to_string())
        .max_results(20)
        .send()
        .await
        .unwrap();
    let mut h: HashMap<String, String> = HashMap::new();
    let apis = res.graphql_apis.unwrap();
    for api in apis {
        h.insert(api.name.unwrap(), api.api_id.unwrap().to_string());
    }
    (h, res.next_token)
}

async fn list_apis(client: &Client) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let r = client.list_graphql_apis().max_results(20).send().await;
    match r {
        Ok(res) => {
            let mut token: Option<String> = res.next_token;

            let apis = res.graphql_apis.unwrap();
            for api in apis {
                h.insert(api.name.unwrap(), api.api_id.unwrap().to_string());
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

pub async fn find_api(client: &Client, name: &str) -> Option<String> {
    let apis = list_apis(client).await;
    apis.get(name).cloned()
}

#[derive(Clone, Debug)]
pub struct Api {
    pub id: String,
    pub https: String,
    pub wss: String,
}

async fn list_graphql_apis(client: &Client) -> HashMap<String, Api> {
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

pub async fn find_graphql_api(client: &Client, name: &str) -> Option<Api> {
    let apis = list_graphql_apis(client).await;
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
    tags: HashMap<String, String>,
) -> (String, HashMap<String, String>) {
    println!("Creating api {}", name.green());
    let auth_type = AuthenticationType::AwsLambda;
    let lambda_auth_config = make_lambda_authorizer(authorizer_arn);
    let additional_auth_type = make_auth_type();
    let r = client
        .create_graphql_api()
        .name(s!(name))
        .set_tags(Some(tags))
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
    _tags: HashMap<String, String>,
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
    tags: HashMap<String, String>,
) -> (String, HashMap<String, String>) {
    let api = find_api(client, name).await;
    match api {
        Some(id) => update_api(client, name, authorizer_arn, &id, tags.clone()).await,
        None => create_api(client, name, authorizer_arn, tags).await,
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

async fn create_type(client: &Client, api_id: &str, _type_name: &str, definition: &str) {
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
                eventbridge::update_datasource(client, api_id, &name, &target_arn, &role_arn).await;
            } else {
                eventbridge::create_datasource(client, api_id, &name, &target_arn, &role_arn).await;
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
        Some(id) => {
            let _ = client.delete_graphql_api().api_id(id).send().await;
        }
        None => (),
    }
}

pub async fn create_types(auth: &Auth, api_id: &str, types: HashMap<String, String>) {
    let client = make_client(auth).await;
    println!("Creating mutation types ({})", &types.len());
    for (t, def) in types {
        create_or_update_type(&client, &api_id, &t, &def).await;
    }
}

pub async fn update_tags(
    client: &Client,
    graphql_arn: &str,
    tags: HashMap<String, String>,
) -> (String, HashMap<String, String>) {
    debug!("Updating tags of api {}", graphql_arn.green());
    let r = client
        .tag_resource()
        .resource_arn(graphql_arn)
        .set_tags(Some(tags.clone()))
        .send()
        .await;

    match r {
        Ok(_res) => {
            let _resp = graphql_arn;
            (graphql_arn.to_string(), tags.clone())
        }
        Err(e) => panic!("{}", e),
    }
}

pub async fn create_events_api(client: &Client, api_name: &str) -> String {
    events::find_or_create_api(client, api_name).await
}

pub async fn create_events_channel(client: &Client, api_id: &str, name: &str, handler: &str) {
    events::create_channel(client, api_id, name, handler).await
}

pub async fn delete_by_id(client: &Client, api_id: &str) {
    println!("Deleting appsync api {}", api_id);
    let res = client
        .delete_graphql_api()
        .api_id(api_id)
        .send()
        .await
        .unwrap();
    println!("{:?}", &res);
}

pub async fn list_api_keys(client: &Client, api_id: &str) -> Vec<String> {
    let res = client.list_api_keys().api_id(api_id).send().await.unwrap();
    match res.api_keys {
        Some(xs) => xs.into_iter().map(|x| x.id.unwrap()).collect(),
        None => vec![],
    }
}

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

pub async fn list_tags(client: &Client, arn: &str) -> Result<HashMap<String, String>, Error> {
    let res = client
        .list_tags_for_resource()
        .resource_arn(arn)
        .send()
        .await;

    match res {
        Ok(r) => {
            let maybe_tags = r.tags();
            match maybe_tags {
                Some(tags) => Ok(tags.clone()),
                None => Ok(HashMap::new()),
            }
        }
        Err(_) => Ok(HashMap::new()),
    }
}

pub type AppsyncClient = Client;
