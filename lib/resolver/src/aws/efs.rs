use authorizer::Auth;
use aws_sdk_efs::{Client, Error};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn get_ap_arn(auth: &Auth, name: &str) -> Result<Option<String>, Error> {
    let client = make_client(auth).await;
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
