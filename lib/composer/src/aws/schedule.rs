use super::{
    role::Role,
    template,
};
use compiler::{
    Entity,
    spec::ScheduleSpec,
};
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Schedule {
    pub group: String,
    pub name: String,
    pub rule_name: String,
    pub target_arn: String,
    pub expression: String,
    pub role_arn: String,
    pub bus: String,
    pub payload: String,
}

fn make_expression(expression: &str) -> String {
    if expression.contains("cron") || expression.contains("rate") {
        String::from(expression)
    } else {
        format!("cron({})", expression)
    }
}

impl Schedule {

    pub fn new(namespace: &str, name: &str, spec: &ScheduleSpec) -> Schedule {

        let rule_name = format!("tc-schedule-{}", &name);
        let payload = &spec.payload.to_string();
        let role_name = Role::entity_role_arn(Entity::Event);

        Schedule {
            group: namespace.to_string(),
            name: name.to_string(),
            rule_name: rule_name,
            target_arn: spec.target.clone(),
            expression: make_expression(&spec.cron),
            role_arn: template::role_arn(&role_name),
            bus: s!("default"),
            payload: payload.to_string(),
        }
    }
}
