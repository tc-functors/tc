use authorizer::Auth;
use aws_sdk_appsync::{
    Client,
    Error,
};
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

async fn list_apis_by_token(client: &Client, token: &str) -> (HashMap<String, String>, Option<String>) {
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
        h.insert(api.name.unwrap(), api.arn.unwrap().to_string() );
    }
    (h, res.next_token)
}


async fn list_apis(client: &Client) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let r = client
        .list_graphql_apis()
        .max_results(20)
        .send().await;
    match r {
        Ok(res) => {
            let mut token: Option<String> = res.next_token;

            let apis = res.graphql_apis.unwrap();
            for api in apis {
                h.insert(api.name.unwrap(), api.arn.unwrap().to_string() );
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

pub async fn find_api(client: &Client, name: &str) -> Option<String> {
    let apis = list_apis(client).await;
    apis.get(name).cloned()
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
