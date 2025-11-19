use super::template;
use compiler::{
    Entity,
    spec::QueueSpec,
};
use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Queue {
    pub name: String,
    pub should_create: bool,
    pub arn: String,
    pub targets: Vec<Target>,
}

impl Queue {
    pub fn new(name: &str, qspec: &QueueSpec) -> Queue {
        let mut targets: Vec<Target> = vec![];
        if let Some(f) = &qspec.function {
            let t = Target {
                entity: Entity::Function,
                name: template::maybe_namespace(&f),
            };
            targets.push(t);
        }


        match &qspec.name {
            Some(n) => Queue {
                name: String::from(n),
                should_create: false,
                arn: template::sqs_arn(&n),
                targets: targets,
            },
            None => Queue {
                name: String::from(name),
                should_create: true,
                arn: template::sqs_arn(&name),
                targets: targets,
            }
        }
    }
}
