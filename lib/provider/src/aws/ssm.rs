use crate::Auth;
use aws_sdk_ssm::{
    Client,
    Error,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn get(client: Client, key: &str) -> Result<String, Error> {
    let r = client
        .get_parameter()
        .name(key)
        .with_decryption(true)
        .send()
        .await;

    let res = match r {
        Ok(v) => v.parameter.unwrap().value.unwrap(),
        Err(_) => String::from(""),
    };

    Ok(res)
}
