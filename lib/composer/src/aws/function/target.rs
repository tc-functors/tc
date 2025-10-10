use configurator::Config;
use compiler::Entity;

use crate::aws::template;
use compiler::FunctionSpec;

use serde_derive::{
    Deserialize,
    Serialize,
};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub arn: String,
    pub shim: Option<String>
}

impl Target {

    pub fn new(namespace: &str,  spec: &FunctionSpec, config: &Config) -> Option<Target> {
        if let Some(tspec) = &spec.target {

            match tspec.entity {
                Entity::Function => {
                    let fqn = template::lambda_fqn(namespace, &tspec.name);
                    Some(Target {
                        entity: Entity::Function,
                        arn: template::lambda_arn(&fqn),
                        shim: None
                    })
                },
                Entity::Event => {
                    let bus = &config.aws.eventbridge.bus;
                    Some(Target {
                        entity: Entity::Event,
                        arn: template::event_bus_arn(&bus),
                        shim: None
                    })
                },
                _ => None
            }
        }  else {
            None
        }
    }
}
