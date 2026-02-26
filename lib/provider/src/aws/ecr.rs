use crate::Auth;
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
                "echo {} | docker login --username AWS --password-stdin {}  2>/dev/null",
                &token,
                get_host(auth)
            );
            u::runcmd_quiet(&cmd, dir);
        }
        None => panic!("Failed to get ECR auth token"),
    }
}

async fn list_images_by_token(
    client: &Client,
    repo: &str,
    token: &str,
) -> (Vec<String>, Option<String>) {
    let res = client
        .list_images()
        .next_token(token)
        .repository_name(repo)
        .send()
        .await
        .unwrap();
    let mut images: Vec<String> = vec![];
    let xs = res.image_ids.unwrap().to_vec();
    for x in xs {
        images.push(x.image_tag.unwrap())
    }
    (images, res.next_token)
}

async fn list_images(auth: &Auth, repo: &str) -> Vec<String> {
    let client = make_client(auth).await;
    let res = client
        .list_images()
        .repository_name(repo)
        .send()
        .await
        .unwrap();
    let mut images: Vec<String> = vec![];
    let initial = res.image_ids.unwrap().to_vec();
    let mut token = res.next_token;

    println!("{:?}", &initial);
    for m in initial {
        if let Some(tag) = m.image_tag {
            images.push(tag);
        }
    }
    match token {
        Some(tk) => {
            token = Some(tk);
            while token.is_some() {
                let (xs, t) = list_images_by_token(&client, repo, &token.unwrap()).await;
                images.extend(xs.clone());
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
    images
}

pub async fn image_exists(auth: &Auth, repo: &str, id: &str) -> bool {
    let images = list_images(auth, repo).await;
    println!("images: {:?}", &images);
    images.contains(&id.to_string())
}
