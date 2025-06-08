use aws_sdk_appsync::{
    Client,
    types::{
        Api,
        AuthMode,
        AuthProvider,
        AuthenticationType,
        EventConfig,
        builders::{
            AuthModeBuilder,
            AuthProviderBuilder,
            EventConfigBuilder,
        },
    },
};
use kit::*;
use std::collections::HashMap;

fn make_auth_provider() -> AuthProvider {
    let b = AuthProviderBuilder::default();
    b.auth_type(AuthenticationType::ApiKey).build().unwrap()
}

fn make_auth_mode() -> AuthMode {
    let b = AuthModeBuilder::default();
    b.auth_type(AuthenticationType::ApiKey).build().unwrap()
}

fn make_event_config() -> EventConfig {
    let b = EventConfigBuilder::default();
    let auth_provider = make_auth_provider();
    let auth_mode = make_auth_mode();
    b.auth_providers(auth_provider)
        .connection_auth_modes(auth_mode.clone())
        .default_publish_auth_modes(auth_mode.clone())
        .default_subscribe_auth_modes(auth_mode)
        .build()
        .unwrap()
}

async fn create_api(client: &Client, name: &str) -> String {
    println!("Creating events api {}", name);
    let r = client
        .create_api()
        .name(s!(name))
        .event_config(make_event_config())
        .send()
        .await;
    match r {
        Ok(res) => {
            let resp = res.api.unwrap();
            resp.api_id.unwrap()
        }
        Err(e) => panic!("{}", e),
    }
}

async fn list_apis(client: &Client) -> HashMap<String, Api> {
    let mut h: HashMap<String, Api> = HashMap::new();
    let r = client.list_apis().send().await;
    match r {
        Ok(res) => {
            let apis = res.apis.unwrap();
            for api in apis {
                h.insert(api.name.clone().unwrap().to_string(), api);
            }
        }
        Err(e) => panic!("{}", e),
    }
    h
}

async fn find_api(client: &Client, name: &str) -> Option<String> {
    let apis = list_apis(client).await;
    match apis.get(name) {
        Some(api) => api.api_id.clone(),
        None => None,
    }
}

async fn create_api_key(client: &Client, api_id: &str) {
    let _ = client
        .create_api_key()
        .api_id(s!(api_id))
        .send()
        .await;
}


pub async fn find_or_create_api(client: &Client, name: &str) -> String {
    match find_api(client, name).await {
        Some(id) => id,
        None => {
            let id = create_api(client, name).await;
            create_api_key(&client, &id).await;
            id
        }
    }
}

pub async fn create_channel(client: &Client, api_id: &str, name: &str, handler: &str) {
    let res = client
        .create_channel_namespace()
        .api_id(s!(api_id))
        .name(s!(name))
        .code_handlers(s!(handler))
        .send()
        .await;
}

async fn get_api_key(client: &Client, api_id: &str) -> Option<String> {
    let res = client.list_api_keys().api_id(s!(api_id)).send().await;
    match res.unwrap().api_keys {
        Some(keys) => keys.into_iter().nth(0).unwrap().id,
        None => None,
    }
}

async fn get_api(client: &Client, api_id: &str) -> Api {
    let res = client.get_api().api_id(s!(api_id)).send().await;
    res.unwrap().api.unwrap()
}

pub struct ApiCred {
    pub api_key: String,
    pub http_domain: String,
}

pub async fn find_api_creds(client: &Client, name: &str) -> Option<ApiCred> {
    let apis = list_apis(client).await;
    match apis.get(name) {
        Some(api) => {
            let api_id = api.api_id.clone().unwrap();
            let details = get_api(client, &api_id).await;
            let api_key = get_api_key(client, &api_id).await;
            let dns = details.dns.unwrap();
            let ac = ApiCred {
                api_key: api_key.unwrap(),
                http_domain: dns.get("HTTP").unwrap().to_string(),
            };
            Some(ac)
        }
        None => None,
    }
}
