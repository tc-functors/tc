mod dynamo;
mod circleci;
pub mod github;
use aws::Env;
use tagger::git;

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

pub async fn create_tag(next: &str, prefix: &str, suffix: &str, push: bool, dry_run: bool) {
    let tag = tagger::next_tag(&prefix, &next, &suffix);
    let has_suffix = suffix != "default";
    if dry_run {
        println!("dry: {:?}", tag);
        tagger::dry_run(&next, tag, has_suffix).await;
    } else {
        tagger::create(&next, tag, push, has_suffix).await;
    }
}

pub fn unwind(prefix: &str) {
    git::fetch_tags();
    let version = git::latest_version(prefix);
    tagger::delete_current_minor(prefix, &version);
}

pub async fn update_metadata(
    env: &Env,
    sandbox: &str,
    service: &str,
    version: &str,
    deploy_env: &str,
    dir: &str
) {
    match std::env::var("TC_UPDATE_METADATA") {
        Ok(_) => {
            if sandbox == "stable" {
                dynamo::put_item(env, service, version, deploy_env, dir).await;
            }
        }
        Err(_) => println!("Not updating metadata")
    }
}
