use anyhow::Result;
use authorizer::Auth;
use aws_config::BehaviorVersion;
use aws_sdk_lambda::{
    Client, config as lambda_config,
    config::retry::RetryConfig,
    primitives::Blob,
    types::{InvocationType, LogType},
};

use base64::{Engine as _, engine::general_purpose};

pub fn make_blob_from_str(payload: &str) -> Blob {
    let buffer = payload.as_bytes();
    Blob::new(buffer)
}

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::from_conf(
        lambda_config::Builder::from(shared_config)
            .behavior_version(BehaviorVersion::latest())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build(),
    )
}

fn print_logs(log_result: Option<String>, payload: Option<Blob>) {
    match log_result {
        Some(x) => {
            let bytes = general_purpose::STANDARD.decode(x).unwrap();
            let logs = String::from_utf8_lossy(&bytes);
            let xs = logs.split("\n").collect::<Vec<&str>>();
            for log in xs {
                if log.contains("error") || log.contains("ERROR") {
                    println!("{}", log);
                } else {
                    println!("{}", log);
                }
            }
        }
        _ => {
            println!("");
        }
    };

    match payload {
        Some(p) => {
            println!("response: {}", String::from_utf8_lossy(&p.into_inner()));
        }
        _ => {
            println!("");
        }
    };
}

pub async fn invoke(client: Client, service: &str, payload: &str) -> Result<()> {
    let blob = make_blob_from_str(payload);
    let r = client
        .invoke()
        .function_name(service)
        .payload(blob)
        .invocation_type(InvocationType::RequestResponse)
        .log_type(LogType::Tail)
        .send()
        .await?;

    print_logs(r.log_result, r.payload);
    Ok(())
}
