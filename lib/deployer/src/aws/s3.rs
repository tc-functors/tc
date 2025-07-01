use aws_sdk_s3::{Client, Error};
use std::path::Path;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::BucketLocationConstraint;
use aws_sdk_s3::types::CreateBucketConfiguration;
use aws_sdk_s3::types::builders::CreateBucketConfigurationBuilder;
use authorizer::Auth;
use walkdir::WalkDir;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(&shared_config)
}

pub async fn put_object(
    client: &Client,
    bucket: &str,
    file: &Path,
    key: &str,
) {
    let body = ByteStream::from_path(file).await;
    let _ = client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body.unwrap())
        .send()
        .await
        .unwrap();
}

pub async fn upload_dir(client: &Client, dir: &str, bucket: &str) {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let f = entry.path();
        if !f.is_dir() {
            let key = f.strip_prefix(dir).unwrap().to_str().unwrap();
            //println!("{} {}", f.display(), &key);
            put_object(client, bucket, &f, &key).await;
        }
    }
}

pub async fn update_bucket_policy(
    client: &Client,
    bucket: &str,
    policy: &str,

) {
    let res = client
        .put_bucket_policy()
        .bucket(bucket)
        .policy(policy)
        .send()
        .await
        .unwrap();
    println!("{:?}", res);
}


async fn bucket_exists(client: &Client, bucket: &str) -> bool {
    let res = client
        .head_bucket()
        .bucket(bucket)
        .send()
        .await;
    match res {
        Ok(_) => true,
        Err(_) => false
    }
}

fn make_bucket_cfg() ->  CreateBucketConfiguration {
    let it = CreateBucketConfigurationBuilder::default();
    it
        .location_constraint(BucketLocationConstraint::UsWest2)
        .build()
}

async fn create_bucket(client: &Client, bucket: &str) {
    let cfg = make_bucket_cfg();
    println!("Creating bucket {}", bucket);
    let res = client
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
