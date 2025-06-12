use super::template;
use crate::{Entity, spec::QueueSpec};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Queue {
    pub name: String,
    pub arn: String,
    pub targets: Vec<Target>,
}

impl Queue {
    pub fn new(name: &str, qspec: &QueueSpec) -> Queue {
        let mut targets: Vec<Target> = vec![];
        if let Some(f) = &qspec.function {
            let t = Target {
                entity: Entity::Function,
                name: f.to_string(),
            };
            targets.push(t);
        }

        Queue {
            name: String::from(name),
            arn: template::sqs_arn(&name),
            targets: targets,
        }
    }
}
