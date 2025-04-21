use aws::Env;
use aws::ecr;

use std::collections::HashMap;
use kit as u;

fn get_host(env: &Env) -> String {
    format!("{}.dkr.ecr.{}.amazonaws.com", env.account(), env.region())
}

pub async fn login(env: &Env, dir: &str) {
    let cmd = format!("AWS_PROFILE={} aws ecr get-login-password --region {} | docker login --username AWS --password-stdin {}",
                      &env.name,
                      env.region(),
                      get_host(env)
    );
    u::run(&cmd, dir);
}

pub async fn publish(env: &Env, image_name: &str) {
    let dir = kit::pwd();
    login(env, &dir).await;
    let cmd = format!("AWS_PROFILE={} docker push {}", &env.name, image_name);
    u::run(&cmd, &dir);
}

pub async fn list(env: &Env, repo: &str) -> HashMap<String, String> {
    let client = ecr::make_client(env).await;
    ecr::list_images(&client, repo).await
}
