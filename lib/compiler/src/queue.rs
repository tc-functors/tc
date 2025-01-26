use serde_derive::{Deserialize, Serialize};
use super::spec::QueueSpec;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Queue {
    pub name: String,
    pub consumer: String,
    pub producer: String
}

impl Queue {

    pub fn new(name: &str, qspec: &QueueSpec) -> Queue {
        Queue {
            name: String::from(name),
            producer: qspec.producer.clone(),
            consumer: qspec.consumer.clone()
        }
    }
}
