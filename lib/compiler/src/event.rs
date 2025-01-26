use kit::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TargetKind {
    Function,
    Mutation,
    StepFunction
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub kind: TargetKind,
    pub id: String,
    pub basename: String,
    pub name: String,
    pub arn: String,
    pub role_arn: String,
    pub input_paths_map: Option<HashMap<String, String>>,
    pub input_template: Option<String>,
}

impl Target {

    fn new(
        kind: TargetKind,
        id: String,
        basename: String,
        input_paths_map: Option<HashMap<String, String>>,
        input_template: Option<String>

    ) -> Target {

        Target {
            kind: kind,
            id: id,
            basename: basename,
            name: format!("{{{{event_target_name}}}}"),
            arn: format!("{{{{event_target_arn}}}}"),
            role_arn: format!("{{{{event_target_arn}}}}"),
            input_paths_map: input_paths_map,
            input_template: input_template
        }
    }
}

pub fn make_targets(
    event_name: &str,
    function: Option<String>,
    mutation: Option<String>,
    stepfunction: Option<String>
) -> Vec<Target> {


    let mut xs: Vec<Target> = vec![];
    if let Some(f) = function {
        let id = format!("{}_lambda_target", event_name);
        let t = Target::new(TargetKind::Function, id, f, None, None);
        xs.push(t);
    }
    if let Some(m) = mutation {

        let id = format!("{}_appsync_target", event_name);
        let mut h: HashMap<String, String> = HashMap::new();
        h.insert(s!("detail"), s!("$.detail"));
        let input_paths_map = Some(h);

        let input_template = Some(format!(r##"{{"detail": <detail>}}"##));
        let t = Target::new(TargetKind::Mutation, id, m, input_paths_map, input_template);
        xs.push(t);
    }
    if let Some(s) = stepfunction {
        let id = format!("{}_target", event_name);
        let t = Target::new(TargetKind::StepFunction, id, s, None, None);
        xs.push(t)
    }

    xs
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Detail {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, Vec<String>>,
}

impl Detail {
    fn new(filter: Option<String>) -> Option<Detail> {
        match filter {
        Some(f) => {
            let d: Detail = serde_json::from_str(&f).unwrap();
            Some(d)
        }
        None => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventPattern {
    #[serde(rename(serialize = "detail-type"))]
    pub detail_type: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
}


impl EventPattern {

    fn new(event_name: &str, source: &str, filter: Option<String>) -> EventPattern {
        let detail = Detail::new(filter);

        EventPattern {
            detail_type: vec![event_name.to_string()],
            source: vec![source.to_string()],
            detail: detail,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub name: String,
    pub rule_name: String,
    pub bus: String,
    pub pattern: EventPattern,
    pub targets: Vec<Target>,
    pub sandboxes: Vec<String>
}

impl Event {

    pub fn new(
        name: &str,
        producer: &str,
        filter: Option<String>,
        pattern: Option<String>,
        targets: Vec<Target>,
        sandboxes: Vec<String>

    ) -> Event {

        let pattern = match pattern {
            Some(p) => {
                let pp: EventPattern = serde_json::from_str(&p).unwrap();
                pp
            },
            None => EventPattern::new(name, producer, filter)
        };

        Event {
            name: s!(name),
            rule_name: format!("{{{{event_rule_name}}}}"),
            bus: format!("{{{{event_bus}}}}"),
            pattern: pattern,
            targets: targets,
            sandboxes: sandboxes
        }
    }
}
