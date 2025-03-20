use aws_sdk_ecr::{Client};

use super::Env;
use std::collections::HashMap;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}


pub async fn put_image(client: &Client, repo: &str, manifest: &str, tag: &str) {
    client
        .put_image()
        .repository_name(repo)
        .image_manifest(manifest)
        .image_tag(tag)
        .send()
        .await
        .unwrap();
}


pub async fn list_images(client: &Client, repository: &str) -> HashMap<String, String> {
    let rsp = client
        .list_images()
        .repository_name(repository)
        .send()
        .await
        .unwrap();

    let images = rsp.image_ids();

    let mut h: HashMap<String, String> = HashMap::new();

    for image in images {
        h.insert(image.image_tag().unwrap().to_string(), image.image_digest().unwrap().to_string());
    }
    h
}
