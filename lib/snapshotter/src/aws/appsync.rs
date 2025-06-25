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

#[derive(Clone, Debug)]
pub struct Api {
    pub arn: String,
}

async fn list_apis(client: &Client) -> HashMap<String, Api> {
    let mut h: HashMap<String, Api> = HashMap::new();
    let r = client.list_graphql_apis().send().await;
    match r {
        Ok(res) => {
            let apis = res.graphql_apis.unwrap();
            for api in apis {
                let a = Api {
                    arn: api.arn.unwrap().to_string(),
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
