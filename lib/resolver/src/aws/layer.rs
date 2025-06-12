use authorizer::Auth;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::{
    Client, Error, config as lambda_config, config::retry::RetryConfig,
    types::LayerVersionsListItem,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        lambda_config::Builder::from(shared_config)
            .behavior_version(BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

fn find_latest(xs: Vec<LayerVersionsListItem>, layer_name: &str) -> String {
    match xs.first() {
        Some(m) => match m.clone().layer_version_arn {
            Some(v) => v,
            _ => panic!("No layer version found"),
        },
        _ => {
            println!("{}: ", layer_name);
            std::panic::set_hook(Box::new(|_| {
                println!("Layer not found");
            }));
            panic!("Layer not found")
        }
    }
}

pub async fn find_version(client: Client, layer_name: &str) -> Result<String, Error> {
    let res = client
        .list_layer_versions()
        .layer_name(layer_name)
        .send()
        .await?;

    match res.layer_versions {
        Some(xs) => Ok(find_latest(xs, layer_name)),
        None => panic!("No layer found"),
    }
}
