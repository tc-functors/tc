use super::Env;
use aws_sdk_efs::{Client, Error};

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

pub async fn get_ap_arn(env: &Env, name: &str) -> Result<Option<String>, Error> {
    let client = make_client(env).await;
    let res = client.describe_access_points().send().await;
    match res {
        Ok(r) => {
            match r.access_points {
                Some(xs) => {
                    for x in xs.iter() {
                        if &x.name.clone().unwrap() == name {
                            return Ok(x.clone().access_point_arn);
                        }
                    }
                }
                None => (),
            }
            return Ok(None);
        }
        Err(e) => panic!("{:?}", e),
    }
}
