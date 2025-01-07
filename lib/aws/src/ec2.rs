use aws_sdk_ec2::types::builders::FilterBuilder;
use aws_sdk_ec2::types::Filter;
use aws_sdk_ec2::{Client, Error};

use super::Env;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

fn make_filter(k: &str, v: &str) -> Filter {
    let f = FilterBuilder::default();
    f.name(k).values(v).build()
}

pub async fn get_subnets(env: &Env, tag: &str) -> Result<Vec<String>, Error> {
    let filter = make_filter(&format!("tag:Name"), tag);
    let client = make_client(env).await;
    let res = client.describe_subnets().filters(filter).send().await;
    match res {
        Ok(r) => {
            let mut ids: Vec<String> = vec![];
            match r.subnets {
                Some(xs) => {
                    for x in xs.iter() {
                        ids.push(x.clone().subnet_id.unwrap())
                    }
                }
                None => (),
            }
            Ok(ids)
        }
        Err(e) => panic!("{:?}", e),
    }
}

pub async fn get_security_groups(env: &Env, tag: &str) -> Result<Vec<String>, Error> {
    let filter = make_filter("group-name", tag);

    let client = make_client(env).await;
    let res = client
        .describe_security_groups()
        .filters(filter)
        .send()
        .await;
    match res {
        Ok(r) => {
            let mut ids: Vec<String> = vec![];
            match r.security_groups {
                Some(xs) => {
                    for x in xs.iter() {
                        ids.push(x.clone().group_id.unwrap())
                    }
                }
                None => (),
            }
            Ok(ids)
        }
        Err(e) => panic!("{:?}", e),
    }
}
