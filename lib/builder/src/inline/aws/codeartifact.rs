use authorizer::Auth;
use aws_sdk_codeartifact::Client;

use kit::*;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn get_auth_token(client: &Client, domain: &str, owner: &str) -> String {
    let res = client
        .get_authorization_token()
        .domain(s!(domain))
        .domain_owner(s!(owner))
        .send()
        .await;
    res.unwrap().authorization_token.unwrap()
}
