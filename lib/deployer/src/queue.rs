use compiler::Queue;
use authorizer::Auth;
use crate::{
    aws::{
        lambda,
        lambda::LambdaClient,
        sqs,
    },
};
use std::collections::HashMap;

async fn create_producer(lambda_client: &LambdaClient, name: &str, sqs_arn: &str) {
    if !name.is_empty() {
        println!("Updating function: {} (producer)", name);
        lambda::update_dlq(lambda_client, name, sqs_arn).await;
    }
}

async fn create_consumer(lambda_client: &LambdaClient, name: &str, sqs_arn: &str) {
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
    let lambda_client = lambda::make_client(&auth).await;
    for (_, queue) in queues {
        println!("Creating queue: {}", &queue.name);
        sqs::create_queue(&client, &queue.name).await;
        let arn = &auth.sqs_arn(&queue.name);
        create_consumer(&lambda_client, &queue.consumer, &arn).await;
        create_producer(&lambda_client, &queue.producer, &arn).await;
    }
}

pub async fn delete(auth: &Auth, queues: &HashMap<String, Queue>) {
    let client = sqs::make_client(&auth).await;
    let lambda_client = lambda::make_client(&auth).await;
    for (_, queue) in queues {
        let arn = &auth.sqs_arn(&queue.name);
        lambda::delete_event_source(&lambda_client, &queue.consumer, &arn).await;
        println!("Deleting queue: {}", &queue.name);
        sqs::delete_queue(&client, &auth.sqs_url(&queue.name)).await;
    }
}
