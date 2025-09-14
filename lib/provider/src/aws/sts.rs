use aws_config::{
    BehaviorVersion,
    SdkConfig,
    environment::credentials::EnvironmentVariableCredentialsProvider,
    sts::AssumeRoleProvider,
};
use aws_smithy_types::retry::RetryConfig;
use aws_sdk_sts::Client;
use std::panic;

// sts

pub async fn make_client(shared_config: &SdkConfig) -> Client {
    Client::new(&shared_config)
}

pub async fn get_account_id(client: &Client) -> String {
    let r = client.get_caller_identity().send().await;

    match r {
        Ok(res) => match res.account {
            Some(acc) => acc,
            None => {
                panic::set_hook(Box::new(|_| {
                    println!(
                        "AWS authentication failed. Please run `aws sso login --profile <profile>"
                    );
                }));
                panic!("Unable to authenticate")
            }
        },
        Err(e) => {
            println!("{:?}", e);
            panic::set_hook(Box::new(|_| {
                println!(
                    "AWS authentication failed. Please run `aws sso login --profile <profile>"
                );
            }));
            panic!("Unable to authenticate")
        }
    }
}


async fn assume_given_role(role_arn: &str) -> SdkConfig {
    let session_name = "TcSession";
    let provider = AssumeRoleProvider::builder(role_arn)
        .session_name(session_name)
        .build_from_provider(EnvironmentVariableCredentialsProvider::new())
        .await;
    aws_config::from_env()
        .retry_config(RetryConfig::adaptive())
        .credentials_provider(provider)
        .behavior_version(BehaviorVersion::latest())
        .load()
        .await
}

pub async fn get_config(profile: &str, assume_role: Option<String>) -> SdkConfig {
    match assume_role {
        Some(role_arn) => assume_given_role(&role_arn).await,
        None => {
            aws_config::from_env()
                .profile_name(profile)
                .retry_config(RetryConfig::adaptive())
                .load()
                .await
        }
    }
}

pub fn get_region() -> String {
    match std::env::var("TC_IGNORE_AWS_VARS") {
        Ok(_) => String::from("us-west-2"),
        Err(_) => match std::env::var("AWS_REGION") {
            Ok(e) => e,
            Err(_) => String::from("us-west-2"),
        },
    }
}
