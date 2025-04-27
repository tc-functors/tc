mod circleci;
mod dynamo;
mod git;
mod github;
mod notifier;
mod router;
mod tagger;

use provider::Env;
pub use router::{
    freeze,
    route,
    unfreeze,
};

pub async fn create_tag(next: &str, prefix: &str, suffix: &str, push: bool, is_dry_run: bool) {
    let tag = tagger::next_tag(&prefix, &next, &suffix);
    let has_suffix = suffix != "default";
    if is_dry_run {
        println!("dry: {:?}", tag);
        tagger::dry_run(&next, tag, has_suffix).await;
    } else {
        tagger::create(&next, tag, push, has_suffix).await;
    }
}

pub fn delete_current_minor(prefix: &str, version: &str) {
    let stable_version = tagger::current_stable_minor(version);
    let tag = format!("{}-{}", &prefix, &stable_version);
    let cmd = format!("git tag -d {} && git push --tag origin :{}", &tag, &tag);
    kit::runcmd_stream(&cmd, &kit::pwd());
}

pub async fn notify(scope: &str, msg: &str) {
    notifier::notify(scope, &notifier::wrap_msg(msg)).await;
}

pub fn changelogs_since_last(prefix: &str, version: &str) -> String {
    let prev_ver = tagger::dec_minor(version);
    let curr_tag = format!("{}-{}", prefix, version);
    let prev_tag = format!("{}-{}", prefix, prev_ver);
    let from_sha = git::tag_revision(&prev_tag);
    let to_sha = git::tag_revision(&curr_tag);
    git::changelogs(&from_sha, &to_sha)
}

pub async fn self_upgrade(repo: &str, tag: Option<String>) {
    github::self_upgrade(repo, tag).await;
}

pub async fn update_metadata(
    env: &Env,
    sandbox: &str,
    service: &str,
    version: &str,
    deploy_env: &str,
    dir: &str,
) {
    match std::env::var("TC_UPDATE_METADATA") {
        Ok(_) => {
            if sandbox == "stable" {
                dynamo::put_item(env, service, version, deploy_env, dir).await;
            }
        }
        Err(_) => println!("Not updating metadata"),
    }
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

pub fn unwind(prefix: &str) {
    git::fetch_tags();
    let version = git::latest_version(prefix);
    tagger::delete_current_minor(prefix, &version);
}

pub fn should_abort(sandbox: &str) -> bool {
    let yes = match std::env::var("CIRCLECI") {
        Ok(_) => false,
        Err(_) => match std::env::var("TC_FORCE_DEPLOY") {
            Ok(_) => false,
            Err(_) => true,
        },
    };
    yes && (sandbox == "stable")
}

pub fn guard(sandbox: &str) {
    if should_abort(sandbox) {
        std::panic::set_hook(Box::new(|_| {
            println!("Cannot create stable sandbox outside CI");
        }));
        panic!("Cannot create stable sandbox outside CI")
    }
}
