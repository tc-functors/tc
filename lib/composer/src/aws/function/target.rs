use crate::aws::template;
use compiler::{
    Entity,
    FunctionSpec,
};
use configurator::Config;
use serde_derive::{
    Deserialize,
    Serialize,
};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub arn: String,
    pub shim: Option<String>,
}

impl Target {

    pub fn make_all(namespace: &str, spec: &FunctionSpec, config: &Config) -> Vec<Target> {

        let mut xs: Vec<Target> = vec![];

        if let Some(tspec) = &spec.target {
            match tspec.entity {
                Entity::Function => {
                    let fqn = template::lambda_fqn(namespace, &tspec.name);
                    let t = Target {
                        entity: Entity::Function,
                        arn: template::lambda_arn(&fqn),
                        shim: None,
                    };
                    xs.push(t);
                }
                Entity::Event => {
                    let bus = &config.aws.eventbridge.bus;
                    let t = Target {
                        entity: Entity::Event,
                        arn: template::event_bus_arn(&bus),
                        shim: None,
                    };
                    xs.push(t);
                }
                _ => (),
            }
        }
        xs
    }
}
