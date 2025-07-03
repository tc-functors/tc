use authorizer::Auth;
use aws_sdk_appsync::Client;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn delete(client: &Client, api_id: &str) {
    println!("Deleting appsync api {}", api_id);
    let res = client
        .delete_graphql_api()
        .api_id(api_id)
        .send()
        .await
        .unwrap();
    println!("{:?}", &res);
}
