use authorizer::Auth;
use aws_sdk_ecr::Client;
use base64::{
    Engine as _,
    engine::general_purpose::URL_SAFE,
};
use kit as u;

fn get_host(auth: &Auth) -> String {
    format!("{}.dkr.ecr.{}.amazonaws.com", auth.account, auth.region)
}

fn get_url(auth: &Auth) -> String {
    format!(
        "https://{}.dkr.ecr.{}.amazonaws.com",
        auth.account, auth.region
    )
}

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

async fn get_auth_token(client: &Client, url: &str) -> Option<String> {
    let res = client.get_authorization_token().send().await.unwrap();
    if let Some(auth_data) = res.authorization_data {
        for x in auth_data.to_vec() {
            if x.proxy_endpoint.unwrap() == url {
                let token = x.authorization_token.unwrap();
                let decoded_bytes = URL_SAFE.decode(token).unwrap();
                let decoded = std::str::from_utf8(&decoded_bytes).unwrap();
                return Some(kit::second(decoded, ":"));
            }
        }
    }
    None
}

pub async fn login(auth: &Auth, dir: &str) {
    let url = get_url(auth);
    let client = make_client(auth).await;
    let maybe_token = get_auth_token(&client, &url).await;

    match maybe_token {
        Some(token) => {
            let cmd = format!(
                "echo {} | docker login --username AWS --password-stdin {}",
                &token,
                get_host(auth)
            );
            u::run(&cmd, dir);
        }
        None => panic!("Failed to get ECR auth token"),
    }
}
