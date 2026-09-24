pub mod aws;

pub use aws::Auth;
use configurator::Config;

pub async fn init(profile: Option<String>, assume_role: Option<String>, region: Option<String>) -> Auth {
    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            let role = match assume_role {
                Some(r) => Some(r),
                None => {
                    let config = Config::new();
                    match profile.clone() {
                        Some(p) => config.ci.roles.get(&p).cloned(),
                        None => panic!("No profile found"),
                    }
                }
            };
            Auth::new(profile.clone(), role, region).await
        }
        Err(_) => Auth::new(profile.clone(), assume_role, region).await,
    }
}

pub async fn init_centralized_auth(given_auth: &Auth, region: Option<String>) -> Auth {
    let config = Config::new();
    let profile = config.aws.lambda.layers_profile.clone();
    let region = match region {
        Some(r) => Some(r),
        None => Some(given_auth.region.clone())
    };

    match profile {
        Some(_) => {
            let cauth = init(profile.clone(), None, region.clone()).await;
            let centralized = cauth
                .assume(profile.clone(), config.role_to_assume(profile), region)
                .await;
            centralized
        }
        None => given_auth.clone(),
    }
}
