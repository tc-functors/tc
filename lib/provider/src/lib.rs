pub mod aws;
pub mod local;

pub use aws::Auth;
use configurator::Config;
use kit as u;

pub async fn init(profile: Option<String>, assume_role: Option<String>) -> Auth {
    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            let role = match assume_role {
                Some(r) => Some(r),
                None => {
                    let config = Config::new(None);
                    let p = u::maybe_string(profile.clone(), "default");
                    config.ci.roles.get(&p).cloned()
                }
            };
            Auth::new(profile.clone(), role).await
        }
        Err(_) => Auth::new(profile.clone(), assume_role).await,
    }
}

pub async fn init_centralized_auth(given_auth: &Auth) -> Auth {
    let config = Config::new(None);
    let profile = config.aws.lambda.layers_profile.clone();
    match profile {
        Some(_) => {
            let cauth = init(profile.clone(), None).await;
            let centralized = cauth
                .assume(profile.clone(), config.role_to_assume(profile))
                .await;
            centralized
        }
        None => given_auth.clone(),
    }
}
