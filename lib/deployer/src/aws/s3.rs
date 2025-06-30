use aws_sdk_lambda::primitives::SdkBody;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Error};
use authorizer::Auth;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(&shared_config)
}

pub async fn upload_dir(client: &Client, dir: &str, bucket: &str) {

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
    println!("{}", res);
}


async fn find_bucket(client: &Client) -> Option<String> {

}

async fn create_bucket(client: &Client, name: &str) {

}

pub async fn find_or_create_bucket(client: &Client, name: &str) {

}
