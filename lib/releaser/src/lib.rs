mod aws;
mod github;
pub mod ci;


pub async fn self_upgrade(repo: &str, tag: Option<String>) {
    github::self_upgrade(repo, tag).await;
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

pub async fn get_release_id(repo: &str, version: Option<String>) -> Option<String> {
    github::get_release_id(repo, version).await
}
