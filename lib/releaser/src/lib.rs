mod circleci;
mod aws;
mod dynamo;
use authorizer::Auth;

// This is a hidden module and not required in core-tc.
// Is there for legacy purpose

fn fetch_tags() {
    kit::sh("git fetch --tags", &kit::pwd());
}

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

pub fn current_repo() -> String {
    kit::sh(
        "basename -s .git `git config --get remote.origin.url`",
        &kit::pwd(),
    )
}

pub async fn update_var(key: &str, val: &str) {
    let repo = current_repo();
    circleci::update_var(&repo, key, val).await;
}

pub async fn release(service: &str, suffix: &str) {
    let repo = current_repo();
    fetch_tags();
    let tag = tagger::next_tag(&service, "minor", &suffix);
    circleci::trigger_release(&repo, &service, &tag.version, &suffix).await;
}

pub async fn deploy(env: &str, service: &str, sandbox: &str, version: &str) {
    let repo = current_repo();
    circleci::trigger_deploy(&repo, &env, &sandbox, &service, &version).await;
}
