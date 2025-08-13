use authorizer::Auth;
use aws_sdk_s3::{
    Client,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(&shared_config)
}

pub async fn get_str(client: &Client, bucket: &str, key: &str) -> String {

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
