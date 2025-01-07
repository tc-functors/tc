use aws_sdk_sqs::types::QueueAttributeName;
use aws_sdk_sqs::Client;
use std::collections::HashMap;

use super::Env;

pub async fn make_client(env: &Env) -> Client {
    let shared_config = env.load().await;
    Client::new(&shared_config)
}

fn make_attributes() -> HashMap<QueueAttributeName, String> {
    let mut m: HashMap<QueueAttributeName, String> = HashMap::new();
    m.insert(QueueAttributeName::VisibilityTimeout, String::from("900"));
    m
}

async fn queue_exists(client: &Client, name: &str) -> bool {
    let r = client
        .get_queue_url()
        .queue_name(String::from(name))
        .send()
        .await;
    match r {
        Ok(res) => match res.queue_url {
            Some(_) => true,
            None => false,
        },
        Err(_) => false,
    }
}

pub async fn create_queue(client: &Client, name: &str) {
    let attrs = make_attributes();
    let exists = queue_exists(client, name).await;
    println!("Checking queue: exists {}", name);
    if !exists {
        let r = client
            .create_queue()
            .queue_name(String::from(name))
            .set_attributes(Some(attrs))
            .send()
            .await;
        match r {
            Ok(_) => (),
            Err(_) => panic!("{:?}", r),
        }
    }
}

pub async fn delete_queue(client: &Client, url: &str) {
    let _ = client
        .delete_queue()
        .queue_url(String::from(url))
        .send()
        .await;
}
