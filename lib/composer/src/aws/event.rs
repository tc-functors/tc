use super::template;
use crate::aws::{
    function::Function,
    mutation::Resolver,
    role::Role,
};
use compiler::{
    entity::Entity,
    spec::EventSpec,
};
use configurator::Config;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub id: String,
    pub name: String,
    pub producer_ns: String,
    pub consumer_ns: String,
    pub arn: String,
    pub role_arn: String,
    pub input_paths_map: Option<HashMap<String, String>>,
    pub input_template: Option<String>,
    pub retry_attempts: Option<i32>,
    pub dead_letter_arn: Option<String>,
    pub maximum_event_age_in_seconds: Option<i32>,
}

impl Target {
    fn new(
        entity: Entity,
        id: &str,
        name: &str,
        arn: &str,
        producer_ns: &str,
        consumer_ns: &str,
        input_paths_map: Option<HashMap<String, String>>,
        input_template: Option<String>,
        retry_attempts: Option<i32>,
        dead_letter_arn: Option<String>,
        maximum_event_age_in_seconds: Option<i32>,
    ) -> Target {
        let abbr_id = if id.chars().count() >= 64 {
            format!("{}-{}", entity.to_str(), &kit::abbreviate(id, "-"))
        } else {
            id.to_string()
        };

        Target {
            entity: entity,
            id: abbr_id,
            name: s!(name),
            producer_ns: s!(producer_ns),
            consumer_ns: s!(consumer_ns),
            arn: s!(arn),
            role_arn: Role::entity_role_arn(Entity::Event),
            input_paths_map: input_paths_map,
            input_template: input_template,
            retry_attempts: retry_attempts,
            dead_letter_arn: dead_letter_arn,
            maximum_event_age_in_seconds: maximum_event_age_in_seconds,
        }
    }
}

fn find_function(f: &str, fns: &HashMap<String, Function>) -> String {
    match fns.get(f) {
        Some(_) => template::maybe_namespace(f),
        None => f.to_string(),
    }
}

fn as_ns(given: &Option<String>, xs: &Vec<String>) -> String {
    match given {
        Some(p) => s!(p),
        None => {
            if xs.len() > 0 {
                let s = xs.into_iter().nth(0).unwrap();
                if s.contains("/") {
                    kit::split_first(s, "/")
                } else {
                    s!(s)
                }
            } else {
                String::from("")
            }
        }
    }
}

pub fn make_targets(
    namespace: &str,
    event_name: &str,
    espec: &EventSpec,
    fallback_fqn: &str,
    fns: &HashMap<String, Function>,
    resolvers: &HashMap<String, Resolver>,
) -> Vec<Target> {
    let EventSpec {
        producer_ns,
        function,
        mutation,
        functions,
        producer,
        state,
        channel,
        ..
    } = espec;

    let dead_letter_arn = espec.dead_letter_queue.as_ref().filter(|q| !q.is_empty()).map(|q| template::sqs_arn(q));
    let retry_attempts = espec.retries;
    let maximum_event_age_in_seconds = espec.maximum_event_age_in_seconds;

    let mut xs: Vec<Target> = vec![];

    let producer_ns = as_ns(producer_ns, producer);
    let consumer_ns = namespace;

    if let Some(f) = function {
        let id = format!("{}_lambda_target", event_name);
        let name = find_function(&f, fns);
        let arn = template::lambda_arn(&name);
        let t = Target::new(
            Entity::Function,
            &id,
            &name,
            &arn,
            &producer_ns,
            &consumer_ns,
            None,
            None,
            retry_attempts,
            dead_letter_arn.clone(),
            maximum_event_age_in_seconds,
        );
        xs.push(t);
    }

    if !functions.is_empty() {
        for f in functions {
            let id = format!("{}_{}_target", event_name, &f);
            let name = find_function(&f, fns);
            let arn = template::lambda_arn(&name);
            let t = Target::new(
                Entity::Function,
                &id,
                &name,
                &arn,
                &producer_ns,
                &consumer_ns,
                None,
                None,
                retry_attempts,
                dead_letter_arn.clone(),
                maximum_event_age_in_seconds,
            );
            xs.push(t);
        }
    }
    if let Some(m) = mutation {
        let id = format!("{}_appsync_target", event_name);
        let mut h: HashMap<String, String> = HashMap::new();
        let input = match resolvers.get(m) {
            Some(m) => match m.input.as_ref() {
                "Event" => s!("$.detail"),
                "EventData" => s!("$.detail.data"),
                "EventDataJSON" => s!("$.detail.data"),
                "EventMetadata" => s!("$.detail.metadata"),
                _ => m.input.clone(),
            },
            None => s!("$.detail"),
        };
        h.insert(s!("detail"), input);

        let input_paths_map = Some(h);
        let input_template = Some(format!(r##"{{"detail": <detail>}}"##));

        let arn = "unresolved";
        let t = Target::new(
            Entity::Mutation,
            &id,
            m,
            &arn,
            &producer_ns,
            &consumer_ns,
            input_paths_map,
            input_template,
            retry_attempts,
            dead_letter_arn.clone(),
            maximum_event_age_in_seconds,
        );
        xs.push(t);
    }
    if let Some(s) = state {
        let id = format!("{}_target", event_name);
        let arn = template::sfn_arn(s);
        let t = Target::new(
            Entity::State,
            &id,
            s,
            &arn,
            &producer_ns,
            &consumer_ns,
            None,
            None,
            retry_attempts,
            dead_letter_arn.clone(),
            maximum_event_age_in_seconds,
        );
        xs.push(t)
    }

    if let Some(c) = channel {
        let id = format!("{}_channel_{}_target", c, event_name);
        let mut h: HashMap<String, String> = HashMap::new();
        h.insert(s!("detail"), s!("$.detail"));
        let input_paths_map = Some(h);
        let arn = format!("{{{{api_destination_arn}}}}");
        let t = Target::new(
            Entity::Channel,
            &id,
            namespace,
            &arn,
            &producer_ns,
            &consumer_ns,
            input_paths_map,
            None,
            retry_attempts,
            dead_letter_arn.clone(),
            maximum_event_age_in_seconds,
        );
        xs.push(t)
    }

    //fallback
    if mutation.is_none()
        && functions.is_empty()
        && function.is_none()
        && state.is_none()
        && channel.is_none()
    {
        let id = format!("{}_target", event_name);
        let arn = template::sfn_arn(fallback_fqn);
        let t = Target::new(
            Entity::State,
            &id,
            fallback_fqn,
            &arn,
            &producer_ns,
            &consumer_ns,
            None,
            None,
            retry_attempts,
            dead_letter_arn.clone(),
            maximum_event_age_in_seconds,
        );
        xs.push(t)
    }
    xs
}

fn parse_detail(filter: Option<String>) -> Option<serde_json::Value> {
    filter.map(|f| serde_json::from_str(&f).unwrap())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventPattern {
    #[serde(rename(serialize = "detail-type", deserialize = "detail-type"))]
    pub detail_type: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
    #[serde(flatten, default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

impl EventPattern {
    fn new(event_name: &str, source: Vec<String>, filter: Option<String>) -> EventPattern {
        let detail = parse_detail(filter);
        let source = source.into_iter().map(serde_json::Value::String).collect();

        EventPattern {
            detail_type: vec![serde_json::Value::String(event_name.to_string())],
            source,
            detail,
            extra: HashMap::new(),
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
        espec: &EventSpec,
        targets: Vec<Target>,
        config: &Config,
        skip: bool,
    ) -> Event {
        let EventSpec {
            rule_name,
            producer,
            filter,
            pattern,
            sandboxes,
            ..
        } = espec;

        let producer = if producer.is_empty() {
            vec![s!("default")]
        } else {
            producer.to_vec()
        };

        let pattern = match pattern {
            Some(p) => {
                let pp: EventPattern = serde_json::from_str(&p).unwrap();
                pp
            }
            None => EventPattern::new(event_name, producer.to_vec(), filter.clone()),
        };

        let bus = match std::env::var("TC_SANDBOXED_EVENTS") {
            Ok(_) => &format!("tc-{{{{sandbox}}}}"),
            Err(_) => &config.aws.eventbridge.bus
        };
        let rule_prefix = &config.aws.eventbridge.rule_prefix;
        let rule_name = match rule_name {
            Some(r) => s!(r),
            None => format!(
                "{}{{{{namespace}}}}-{}-{{{{sandbox}}}}",
                rule_prefix,
                s!(event_name)
            ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_simple_filter() {
        let json = r#"{"detail-type":["OrderCreated"],"source":["myapp"],"detail":{"metadata":{"type":["foo"]}}}"#;
        let pattern: EventPattern = serde_json::from_str(json).unwrap();

        assert_eq!(
            pattern.detail_type,
            vec![serde_json::Value::from("OrderCreated")]
        );
        assert_eq!(pattern.source, vec![serde_json::Value::from("myapp")]);
        assert!(pattern.detail.is_some());

        let out = serde_json::to_string(&pattern).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(reparsed, original);
    }

    #[test]
    fn roundtrip_or_with_anything_but_and_exists() {
        let json = r#"{"detail-type":["StateChange"],"source":["myapp"],"detail":{"$or":[{"state":[{"anything-but":"initializing"}]},{"state":[{"exists":false}]}]}}"#;
        let pattern: EventPattern = serde_json::from_str(json).unwrap();

        assert_eq!(
            pattern.detail_type,
            vec![serde_json::Value::from("StateChange")]
        );
        assert_eq!(pattern.source, vec![serde_json::Value::from("myapp")]);
        assert!(pattern.detail.is_some());

        let detail = pattern.detail.as_ref().unwrap();
        assert!(detail.get("$or").is_some(), "$or must be preserved in detail");

        let out = serde_json::to_string(&pattern).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(reparsed, original);
    }

    #[test]
    fn roundtrip_prefix_and_numeric() {
        let json = r#"{"detail-type":["Metric"],"detail":{"name":[{"prefix":"cpu."}],"value":[{"numeric":[">",0,"<=",100]}]}}"#;
        let pattern: EventPattern = serde_json::from_str(json).unwrap();

        let out = serde_json::to_string(&pattern).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(reparsed, original);
    }

    #[test]
    fn roundtrip_full_pattern_passthrough() {
        let json = r#"{"detail-type":["StateChange"],"source":["myapp"],"account":["123456789012"],"region":["us-east-1"],"detail":{"$or":[{"state":[{"anything-but":"initializing"}]},{"state":[{"exists":false}]}]}}"#;
        let pattern: EventPattern = serde_json::from_str(json).unwrap();

        assert!(pattern.extra.contains_key("account"));
        assert!(pattern.extra.contains_key("region"));

        let out = serde_json::to_string(&pattern).unwrap();
        let reparsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(reparsed, original);
    }

    #[test]
    fn filter_as_detail_value() {
        let filter = r#"{"$or":[{"state":[{"anything-but":"initializing"}]},{"state":[{"exists":false}]}]}"#;
        let detail = parse_detail(Some(filter.to_string()));
        assert!(detail.is_some());
        assert!(detail.as_ref().unwrap().get("$or").is_some());
    }

    #[test]
    fn new_builds_pattern_with_complex_filter() {
        let filter = r#"{"$or":[{"state":[{"anything-but":"initializing"}]},{"state":[{"exists":false}]}]}"#;
        let pattern = EventPattern::new("MyEvent", vec!["myapp".into()], Some(filter.to_string()));

        assert_eq!(
            pattern.detail_type,
            vec![serde_json::Value::from("MyEvent")]
        );
        assert_eq!(pattern.source, vec![serde_json::Value::from("myapp")]);
        assert!(pattern.detail.as_ref().unwrap().get("$or").is_some());

        let out = serde_json::to_string(&pattern).unwrap();
        assert!(out.contains(r#""$or""#));
        assert!(out.contains(r#""anything-but""#));
        assert!(out.contains(r#""exists""#));
    }
}
