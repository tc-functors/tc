use super::Env;
use aws_sdk_sts::Client;
use std::panic;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

pub async fn get_account_id(client: &Client) -> String {
    let r = client.get_caller_identity().send().await;

    match r {
        Ok(res) => {
            match res.account {
                Some(acc) => acc,
                None => {
                    panic::set_hook(Box::new(|_| {
                        println!("AWS authentication failed. Please run `aws sso login --profile <profile>");
                    }));
                    panic!("Unable to authenticate")
                }
            }
        }
        Err(_) => {
            panic::set_hook(Box::new(|_| {
                println!(
                    "AWS authentication failed. Please run `aws sso login --profile <profile>"
                );
            }));
            panic!("Unable to authenticate")
        }
    }
}
