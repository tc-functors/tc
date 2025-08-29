mod circleci;

// This is a hidden module and not required in core-tc.
// Is there for legacy purpose

fn fetch_tags() {
    kit::sh("git fetch --tags", &kit::pwd());
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

pub async fn release(service: &str, suffix: &str, tag: &str) -> String {
    let repo = current_repo();
    fetch_tags();
    let url = circleci::trigger_release(&repo, &service, &tag, &suffix).await;
    url
}

pub async fn deploy(env: &str, service: &str, sandbox: &str, version: &str) -> String {
    let repo = current_repo();
    circleci::trigger_tag(&repo, &env, &sandbox, &service, &version).await
}

pub async fn deploy_branch(env: &str, service: &str, sandbox: &str, branch: &str) -> String {
    let repo = current_repo();
    circleci::trigger_branch(&repo, &env, &sandbox, &service, branch).await
}

pub async fn build(service: &str, function: &str, branch: &str) -> String {
    let repo = current_repo();
    circleci::trigger_build(&repo, service, function, branch).await

}
