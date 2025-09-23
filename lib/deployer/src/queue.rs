use compiler::Entity;
use composer::{
    Queue,
};
use provider::{
    Auth,
    aws::{
        lambda,
        sqs,
    },
};
use std::collections::HashMap;

async fn _create_lambda_producer(auth: &Auth, name: &str, sqs_arn: &str) {
    let lambda_client = lambda::make_client(&auth).await;
    if !name.is_empty() {
        println!("Updating function: {} (producer)", name);
        lambda::_update_dlq(&lambda_client, name, sqs_arn).await;
    }
}

async fn create_lambda_consumer(auth: &Auth, name: &str, sqs_arn: &str) {
    let lambda_client = lambda::make_client(&auth).await;
    println!("Updating function: {} (consumer)", name);
    lambda::create_event_source(&lambda_client, name, &sqs_arn).await;
    let _ = lambda::add_permission(
        lambda_client.clone(),
        name,
        "sns.amazonaws.com",
        &sqs_arn,
        "sqs-permission",
    )
    .await;
    lambda::update_event_invoke_config(&lambda_client, name).await;
}

pub async fn create(auth: &Auth, queues: &HashMap<String, Queue>) {
    let client = sqs::make_client(&auth).await;
    for (_, queue) in queues {
        sqs::create_queue(&client, &queue.name).await;
        for target in &queue.targets {
            println!("Creating queue: {}", &queue.name);
            match target.entity {
                Entity::Function => create_lambda_consumer(auth, &target.name, &queue.arn).await,
                _ => (),
            }
        }
    }
}

async fn delete_lambda_consumer(auth: &Auth, name: &str, queue_arn: &str) {
    let lambda_client = lambda::make_client(&auth).await;
    lambda::delete_event_source(&lambda_client, name, queue_arn).await
}

pub async fn delete(auth: &Auth, queues: &HashMap<String, Queue>) {
    let client = sqs::make_client(&auth).await;
    for (_, queue) in queues {
        for target in &queue.targets {
            match target.entity {
                Entity::Function => delete_lambda_consumer(auth, &target.name, &queue.arn).await,
                _ => (),
            }
        }
        println!("Deleting queue: {}", &queue.name);
        sqs::delete_queue(&client, &auth.sqs_url(&queue.name)).await;
    }
}

pub async fn update(_auth: &Auth, _queues: &HashMap<String, Queue>, _c: &str) {
    todo!()
}
