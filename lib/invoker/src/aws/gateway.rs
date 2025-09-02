use authorizer::Auth;
use aws_sdk_apigatewayv2::Client;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn find_api_id(client: &Client, name: &str) -> Option<String> {
    let r = client
        .get_apis()
        .max_results(String::from("1000"))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(apis) => {
            for api in apis.to_vec() {
                match api.name {
                    Some(n) => {
                        if n == name {
                            return api.api_id;
                        }
                    }
                    None => (),
                }
            }
            return None;
        }
        None => None,
    }
}
