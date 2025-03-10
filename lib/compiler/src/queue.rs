use serde_derive::{Deserialize, Serialize};
use super::spec::QueueSpec;
use super::template;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Queue {
    pub name: String,
    pub arn: String,
    pub consumer: String,
    pub producer: String
}

impl Queue {

    pub fn new(name: &str, qspec: &QueueSpec) -> Queue {
        Queue {
            name: String::from(name),
            arn: template::sqs_arn(&name),
            producer: qspec.producer.clone(),
            consumer: qspec.consumer.clone()
        }
    }
}
