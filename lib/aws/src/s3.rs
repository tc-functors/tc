use aws_sdk_lambda::primitives::SdkBody;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Error};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};

use super::Env;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

pub async fn put_file(
    client: Client,
    bucket: &str,
    file_name: &str,
    key: &str,
) -> Result<(), Error> {
    let body = ByteStream::from_path(Path::new(file_name)).await;
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.unwrap())
        .send()
        .await
        .unwrap();
    Ok(())
}

pub async fn _put_str(
    client: Client,
    bucket: &str,
    data_str: &str,
    key: &str,
) -> Result<(), Error> {
    let body = ByteStream::new(SdkBody::from(data_str));
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await
        .unwrap();
    Ok(())
}

pub async fn get_object(client: Client, bucket: &str, key: &str) -> Result<String, Error> {
    let res = client.get_object().bucket(bucket).key(key).send().await;

    match res {
        Ok(r) => {
            let stream = r.body;
            let mut lines = BufReader::new(stream.into_async_read()).lines();
            let mut s: String = String::from("");
            while let Some(line) = lines.next_line().await.unwrap() {
                s = line;
            }
            Ok(s)
        }

        Err(_) => {
            std::panic::set_hook(Box::new(|_| {
                println!("id not found");
            }));
            panic!("s3://{bucket}/{key} not found");
        }
    }
}

pub async fn list_keys(client: Client, bucket: &str, prefix: &str) -> Vec<String> {
    let r = client
        .list_objects()
        .bucket(bucket)
        .delimiter("/")
        .prefix(prefix)
        .send()
        .await
        .unwrap();
    let mut keys: Vec<String> = vec![];
    match r.contents {
        Some(c) => {
            for obj in c {
                let key = obj.key.unwrap();
                keys.push(key);
            }
        }
        None => println!(""),
    }
    keys
}
