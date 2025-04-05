use kit::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::spec::Consumes;
use super::template;
use configurator::Config;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TargetKind {
    Function,
    Mutation,
    StepFunction
}

impl TargetKind {

    pub fn to_str(&self) -> String {
        match self {
            TargetKind::Function => s!("function"),
            TargetKind::Mutation => s!("appsync"),
            TargetKind::StepFunction => s!("sfn")
        }
    }
}

fn as_ns(given: &Option<String>, s: &str) -> String {
    match given {
        Some(p) => s!(p),
        None => {
            if s.contains("/") {
                kit::split_first(s, "/")
            } else {
                s!(s)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub kind: TargetKind,
    pub id: String,
    pub name: String,
    pub producer_ns: String,
    pub consumer_ns: String,
    pub arn: String,
    pub role_arn: String,
    pub input_paths_map: Option<HashMap<String, String>>,
    pub input_template: Option<String>,
}

impl Target {

    fn new(
        kind: TargetKind,
        id: &str,
        name: &str,
        arn: &str,
        producer_ns: &str,
        consumer_ns: &str,
        input_paths_map: Option<HashMap<String, String>>,
        input_template: Option<String>,


    ) -> Target {

        let abbr_id = if id.chars().count() >= 64 {
            format!("{}-{}", kind.to_str(), &kit::abbreviate(id, "-"))
        } else {
            id.to_string()
        };

        Target {
            kind: kind,
            id: abbr_id,
            name: s!(name),
            producer_ns: s!(producer_ns),
            consumer_ns: s!(consumer_ns),
            arn: s!(arn),
            role_arn: template::role_arn("tc-base-event-role"),
            input_paths_map: input_paths_map,
            input_template: input_template
        }
    }
}

pub fn make_targets(
    namespace: &str,
    event_name: &str,
    consumes: &Consumes,
    fallback_fqn: &str
) -> Vec<Target> {

    let Consumes {
        producer_ns,
        producer,
        function,
        mutation,
        functions,
        stepfunction,
        ..
    } = consumes;

    let mut xs: Vec<Target> = vec![];

    let producer_ns = as_ns(producer_ns, producer);
    let consumer_ns = namespace;

    if let Some(ref f) = function {
        let id = format!("{}_lambda_target", event_name);
        let arn = template::lambda_arn(&f);
        let t = Target::new(
            TargetKind::Function,
            &id,
            &f,
            &arn,
            &producer_ns,
            &consumer_ns,
            None,
            None
        );
        xs.push(t);
    }

    if !functions.is_empty()  {
        for f in functions {
            let id = format!("{}_{}_target", event_name, &f);
            let arn = template::lambda_arn(&f);
            let t = Target::new(
                TargetKind::Function,
                &id,
                &f,
                &arn,
                &producer_ns,
                &consumer_ns,
                None,
                None
            );
            xs.push(t);
        }
    }
    if let Some(ref m) = mutation {

        let id = format!("{}_appsync_target", event_name);
        let mut h: HashMap<String, String> = HashMap::new();
        h.insert(s!("detail"), s!("$.detail"));

        let input_paths_map = Some(h);
        let input_template = Some(format!(r##"{{"detail": <detail>}}"##));

        let arn = "unresolved";
        let t = Target::new(
            TargetKind::Mutation,
            &id,
            m,
            &arn,
            &producer_ns,
            &consumer_ns,
            input_paths_map,
            input_template
        );
        xs.push(t);
    }
    if let Some(ref s) = stepfunction {
        let id = format!("{}_target", event_name);
        let arn = template::sfn_arn(s);
        let t = Target::new(
            TargetKind::StepFunction,
            &id,
            s,
            &arn,
            &producer_ns,
            &consumer_ns,
            None,
            None
        );
        xs.push(t)
    }

    //fallback
    if mutation.is_none() && function.is_none() && stepfunction.is_none() {
        let id = format!("{}_target", event_name);
        let arn = template::sfn_arn(fallback_fqn);
        let t = Target::new(
            TargetKind::StepFunction,
            &id,
            fallback_fqn,
            &arn,
            &producer_ns,
            &consumer_ns,
            None,
            None
        );
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
    #[serde(rename(serialize = "detail-type", deserialize = "detail-type"))]
    pub detail_type: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
}

impl EventPattern {

    fn new(event_name: &str, source: &str, filter: Option<String>) -> EventPattern {
        let detail = Detail::new(filter);

        let source = if source.contains("/") {
            kit::split_last(source, "/")
        } else {
            s!(source)
        };

        EventPattern {
            detail_type: vec![event_name.to_string()],
            source: vec![source],
            detail: detail,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub skip: bool,
    pub name: String,
    pub rule_name: String,
    pub bus: String,
    pub bus_arn: String,
    pub pattern: EventPattern,
    pub targets: Vec<Target>,
    pub sandboxes: Vec<String>,
}

impl Event {

    pub fn new(
        event_name: &str,
        consumer_spec: &Consumes,
        targets: Vec<Target>,
        config: &Config,
        skip: bool
    ) -> Event {

        let Consumes {
            rule_name,
            producer,
            filter,
            pattern,
            sandboxes,
            ..
        } = consumer_spec;


        let pattern = match pattern {
            Some(p) => {
                let pp: EventPattern = serde_json::from_str(&p).unwrap();
                pp
            },
            None => EventPattern::new(event_name, producer, filter.clone())
        };

        let bus = &config.aws.eventbridge.bus;
        let rule_prefix = &config.aws.eventbridge.rule_prefix;
        let rule_name = match rule_name {
            Some(r) => s!(r),
            None => format!("{}{{{{namespace}}}}-{}-{{{{sandbox}}}}",
                            rule_prefix, s!(event_name))
        };


        Event {
            skip: skip,
            name: s!(event_name),
            rule_name: rule_name,
            bus: bus.clone(),
            bus_arn: template::event_bus_arn(bus),
            pattern: pattern,
            targets: targets,
            sandboxes: sandboxes.clone(),
        }
    }
}
