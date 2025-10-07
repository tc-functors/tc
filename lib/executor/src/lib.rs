mod circleci;

// this is a module that abstracts remote execution of tc commands in a remote executor

// enum Executor {
//     CircleCI,
//     Github,
//     Drone,
//     Rebar
// }

fn fetch_tags() {
    kit::sh("git fetch --tags", &kit::pwd());
}

pub fn current_repo() -> String {
    kit::sh(
        "basename -s .git `git config --get remote.origin.url`",
        &kit::pwd(),
    )
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

pub async fn deploy_snapshot(env: &str, sandbox: &str, snapshot: &str) -> String {
    let repo = current_repo();
    circleci::trigger_pipeline(&repo, env, sandbox, snapshot).await
}

pub async fn create(env: &str, sandbox: &str, dir: &str, branch: &str) -> String {
    let repo = current_repo();
    circleci::trigger_create(&repo, &env, &sandbox, dir, branch).await
}

pub async fn update(env: &str, sandbox: &str, dir: &str, branch: &str) -> String {
    let repo = current_repo();
    circleci::trigger_update(&repo, &env, &sandbox, dir, branch).await
}

pub async fn build(service: &str, function: &str, branch: &str) -> String {
    let repo = current_repo();
    circleci::trigger_build(&repo, service, function, branch).await
}
