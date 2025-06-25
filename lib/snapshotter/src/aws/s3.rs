use aws_sdk_lambda::primitives::SdkBody;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Error};
use authorizer::Auth;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(&shared_config)
}

pub async fn put_str(
    client: &Client,
    bucket: &str,
    key: &str,
    payload: &str,
) -> Result<(), Error> {
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
        Err(e) => panic!("{}", e)
    }
}

pub fn parts_of(uri: &str) -> (String, String) {
    let uri = s3uri::from_uri(uri).unwrap();
    (uri.bucket, uri.key.unwrap().to_string())
}
