use aws_sdk_ssm::types::ParameterType;
use aws_sdk_ssm::{Client, Error};

use super::Env;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

pub async fn put(client: Client, key: &str, val: &str) -> Result<(), Error> {
    client
        .put_parameter()
        .overwrite(true)
        .r#type(ParameterType::SecureString)
        .name(key)
        .value(val)
        .description(val)
        .send()
        .await?;
    Ok(())
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

pub async fn _get1(client: Client, key: &str) -> String {
    let res = client.get_parameter().name(key).send().await.unwrap();
    res.parameter.unwrap().value.unwrap()
}
