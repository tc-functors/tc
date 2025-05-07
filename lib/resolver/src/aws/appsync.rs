use authorizer::Auth;
use aws_sdk_appsync::{
    Client,
};
use kit::*;
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

async fn list_apis(client: &Client) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let r = client.list_graphql_apis().send().await;
    match r {
        Ok(res) => {
            let apis = res.graphql_apis.unwrap();
            for api in apis {
                h.insert(api.name.unwrap(), api.api_id.unwrap().to_string());
            }
        }
        Err(e) => panic!("{}", e),
    }
    h
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

pub async fn find_api(client: &Client, name: &str) -> Option<String> {
    let apis = list_apis(client).await;
    apis.get(name).cloned()
}
