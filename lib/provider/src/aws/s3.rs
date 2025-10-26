use crate::Auth;
use aws_sdk_lambda::primitives::SdkBody;
pub use aws_sdk_s3::Client;
use aws_sdk_s3::{
    Error,
    primitives::ByteStream,
    types::{
        BucketLocationConstraint,
        CreateBucketConfiguration,
        builders::CreateBucketConfigurationBuilder,
    },
};
use kit::*;
use std::path::Path;
use walkdir::WalkDir;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(&shared_config)
}

fn as_content_type(key: &str) -> String {
    let ext = kit::split_last(key, ".");
    match ext.as_ref() {
        "html" => s!("text/html"),
        "js" => s!("text/javascript"),
        "css" => s!("text/css"),
        "xml" => s!("text/xml"),
        "json" => s!("application/json"),
        "pdf" => s!("application/pdf"),
        "png" => s!("image/png"),
        "gif" => s!("image/gif"),
        "jpg" | "jpeg" => s!("image/jpeg"),
        _ => s!("application/octet-stream"),
    }
}

pub async fn put_str(client: &Client, bucket: &str, key: &str, payload: &str) -> Result<(), Error> {
    let body = ByteStream::new(SdkBody::from(payload));
    let res = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await;

    match res {
        Ok(_) => Ok(()),
        Err(e) => panic!("{}", e),
    }
}

pub async fn put_object(client: &Client, bucket: &str, file: &Path, key: &str) {
    let body = ByteStream::from_path(file).await;
    let ctype = as_content_type(key);
    let _ = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.unwrap())
        .content_type(ctype)
        .send()
        .await
        .unwrap();
}

pub async fn upload_dir(client: &Client, dir: &str, bucket: &str, prefix: &str) {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let f = entry.path();
        if !f.is_dir() {
            let part_key = f.strip_prefix(dir).unwrap().to_str().unwrap();
            let key = format!("{}/{}", prefix, &part_key);
            //println!("{} {}", f.display(), &key);
            put_object(client, bucket, &f, &key).await;
        }
    }
}

pub async fn update_bucket_policy(client: &Client, bucket: &str, policy: &str) {
    let _ = client
        .put_bucket_policy()
        .bucket(bucket)
        .policy(policy)
        .send()
        .await
        .unwrap();
}

pub async fn get_bucket_policy(client: &Client, bucket: &str) -> Option<String> {
    let res = client.get_bucket_policy().bucket(bucket).send().await;
    match res {
        Ok(_) => res.unwrap().policy,
        Err(_) => None,
    }
}

async fn bucket_exists(client: &Client, bucket: &str) -> bool {
    let res = client.head_bucket().bucket(bucket).send().await;
    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn make_bucket_cfg() -> CreateBucketConfiguration {
    let it = CreateBucketConfigurationBuilder::default();
    it.location_constraint(BucketLocationConstraint::UsWest2)
        .build()
}

async fn create_bucket(client: &Client, bucket: &str) {
    let cfg = make_bucket_cfg();
    println!("Creating bucket {}", bucket);
    let _ = client
        .create_bucket()
        .bucket(bucket)
        .create_bucket_configuration(cfg)
        .send()
        .await
        .unwrap();
}

pub async fn find_or_create_bucket(client: &Client, bucket: &str) {
    if !bucket_exists(client, bucket).await {
        create_bucket(client, bucket).await
    }
}

pub async fn get_str(client: &Client, bucket: &str, key: &str) -> String {
    tracing::debug!("s3:getObjct s3://{}/{}", bucket, key);
    let result = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .response_content_type("application/json")
        .send()
        .await
        .unwrap();

    let bytes = result.body.collect().await.unwrap().into_bytes();
    std::str::from_utf8(&bytes).unwrap().to_string()
}

pub fn parts_of(uri: &str) -> (String, String) {
    let uri = s3uri::from_uri(uri).unwrap();
    (uri.bucket, uri.key.unwrap().to_string())
}

pub async fn list_keys(client: &Client, bucket: &str, prefix: &str) -> Vec<String> {
    tracing::debug!("s3:list s3://{}/{}", bucket, prefix);
    let result = client
        .list_objects()
        .bucket(bucket)
        .prefix(prefix)
        .send()
        .await
        .unwrap();

    let res = result.contents.unwrap();
    let mut xs: Vec<String> = vec![];
    for x in res {
        xs.push(x.key.unwrap());
    }
    xs
}
