use aws::Env;
use aws::ecr;

use std::collections::HashMap;
use kit as u;

fn get_host(env: &Env) -> String {
    format!("{}.dkr.ecr.{}.amazonaws.com", env.account(), env.region())
}

fn is_logged_in() -> bool {
    // let data = u::slurp(&u::expand_path("~/.docker/config"));
    // let res: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    // println!("{}", &res["auths"]);
    true
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
    if !is_logged_in() {
        login(env, &dir).await;
    }
    let cmd = format!("docker push {}", image_name);
    u::run(&cmd, &dir);
}

pub async fn list(env: &Env, repo: &str) -> HashMap<String, String> {
    let client = ecr::make_client(env).await;
    ecr::list_images(&client, repo).await
}
