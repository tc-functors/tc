use authorizer::Auth;
use aws_sdk_apigatewayv2::{
    Client,
};
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

async fn list_apis_by_token(client: &Client, token: &str) -> (HashMap<String, HashMap<String, String>>, Option<String>) {
    let res = client
        .get_apis()
        .next_token(token.to_string())
        .send()
        .await
        .unwrap();
    let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
    let apis = res.items.unwrap();
    for api in apis {
        h.insert(api.name.unwrap(), api.tags.unwrap());
    }
    (h, res.next_token)
}


async fn list_apis(client: &Client) -> HashMap<String, HashMap<String, String>> {
    let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
    let r = client
        .get_apis()
        .send().await;
    match r {
        Ok(res) => {
            let mut token: Option<String> = res.next_token;

            let apis = res.items.unwrap();
            for api in apis {
                h.insert(api.name.unwrap(), api.tags.unwrap());
            }

            match token {
                Some(tk) => {
                    token = Some(tk);
                    while token.is_some() {
                        let (xs, t) =
                            list_apis_by_token(client, &token.unwrap()).await;
                        h.extend(xs.clone());
                        token = t.clone();
                        if let Some(x) = t {
                            if x.is_empty() {
                                break;
                            }
                        }
                    }
                },
                None => ()
            }
        }
        Err(e) => panic!("{}", e),
    }
    h
}

pub async fn find_tags(client: &Client, name: &str) -> HashMap<String, String> {
    let apis = list_apis(client).await;
    let maybe_h  = apis.get(name);
    match maybe_h {
        Some(p) => p.clone(),
        None => HashMap::new()
    }
}
