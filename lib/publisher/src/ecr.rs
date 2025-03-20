use aws::Env;
use aws::ecr;

use std::collections::HashMap;


pub async fn publish(_env: &Env, _name: &str) {

}

pub async fn list(env: &Env, repo: &str) -> HashMap<String, String> {
    let client = ecr::make_client(env).await;
    ecr::list_images(&client, repo).await
}
