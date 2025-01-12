use serde_derive::{Deserialize, Serialize};

use super::{Context, Topology};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Schedule {
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

pub fn make(context: &Context, topology: &Topology) -> Vec<Schedule> {
    let Context { env, .. } = context;
    let Topology { schedules, .. } = topology;
    let role_name = env.base_role("event");
    let role_arn = &env.role_arn(&role_name);

    let mut xs: Vec<Schedule> = vec![];

    for (name, schedule) in schedules {
        let rule_name = format!("tc-schedule-{}", &name);
        let payload = &schedule.payload.to_string();
        let s = Schedule {
            name: name.to_string(),
            rule_name: rule_name,
            target_arn: context.render(&schedule.target),
            expression: context.render(&make_expression(&schedule.cron)),
            role_arn: role_arn.to_owned(),
            bus: String::from("default"),
            payload: context.render(&payload),
        };
        xs.push(s);
    }
    xs
}
