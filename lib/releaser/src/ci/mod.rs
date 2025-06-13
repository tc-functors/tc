mod circleci;
pub mod dynamo;
use crate::{
    git,
    tagger,
};
use authorizer::Auth;

pub async fn update_metadata(
    auth: &Auth,
    sandbox: &str,
    service: &str,
    version: &str,
    deploy_env: &str,
    dir: &str,
) {
    match std::env::var("TC_UPDATE_METADATA") {
        Ok(_) => {
            if sandbox == "stable" {
                dynamo::put_item(auth, service, version, deploy_env, dir).await;
            }
        }
        Err(_) => println!("Not updating metadata"),
    }
}

pub async fn update_var(key: &str, val: &str) {
    let repo = git::current_repo();
    circleci::update_var(&repo, key, val).await;
}

pub async fn release(service: &str, suffix: &str) {
    let repo = git::current_repo();
    git::fetch_tags();
    let tag = tagger::next_tag(&service, "minor", &suffix);
    circleci::trigger_release(&repo, &service, &tag.version, &suffix).await;
}

pub async fn deploy(env: &str, service: &str, sandbox: &str, version: &str) {
    let repo = git::current_repo();
    circleci::trigger_deploy(&repo, &env, &sandbox, &service, &version).await;
}
