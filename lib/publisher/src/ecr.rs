use aws::Env;
use aws::ecr;

use std::collections::HashMap;
use serde_derive::{Deserialize};
use base64::{engine::general_purpose, Engine as _};
use kit as u;

fn get_host(env: &Env) -> String {
    format!("{}.dkr.ecr.{}.amazonaws.com", env.account(), env.region())
}

#[derive(Deserialize, Clone, Debug)]
struct Auth {
    auth: String
}

type Config = HashMap<String, HashMap<String, Auth>>;

fn is_logged_in(env: &Env) -> bool {
    let cfg_file = &u::expand_path("~/.docker/config.json");
    if !u::file_exists(cfg_file) {
        return false
    }
    match std::env::var("TC_SKIP_ECR_LOGIN") {
        Ok(_) => true,
        Err(_) => {
            let data = u::slurp(cfg_file);
            let config: Config = serde_json::from_str(&data).unwrap();
            let host = get_host(env);
            let maybe_auth = match config.get("auths") {
                Some(x) => match x.get(&host) {
                    Some(g) => Some(g.auth.clone()),
                    None => None
                },
                None => None
            };

            match maybe_auth {
                Some(auth) => {
                    let bytes1 = general_purpose::STANDARD.decode(auth).unwrap();
                    let sa1 = &String::from_utf8_lossy(&bytes1)[4..];
                    let bytes2 = general_purpose::STANDARD.decode(sa1).unwrap();
                    let sa = String::from_utf8_lossy(&bytes2);
                    let val: serde_json::Value = serde_json::from_str(&sa).unwrap();
                    let exp = &val["expiration"].as_i64().unwrap();
                    let now = u::current_millis() / 1000;
                    (exp - now) > 60
                },
                None => false
            }
        }
    }
}

pub async fn login(env: &Env, dir: &str) {
    let cmd = format!("aws ecr get-login-password --region {} | docker login --username AWS --password-stdin {}",
                      env.region(),
                      get_host(env)
    );
    u::run(&cmd, dir);
}

pub async fn publish(env: &Env, image_name: &str) {
    let dir = kit::pwd();
    if !is_logged_in(env) {
        login(env, &dir).await;
    }
    let cmd = format!("docker push {}", image_name);
    u::run(&cmd, &dir);
}

pub async fn list(env: &Env, repo: &str) -> HashMap<String, String> {
    let client = ecr::make_client(env).await;
    ecr::list_images(&client, repo).await
}
