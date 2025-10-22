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
        );
        xs.push(t)
    }

    //fallback
    if mutation.is_none() && function.is_none() && state.is_none() && channel.is_none() {
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
    fn new(event_name: &str, source: Vec<String>, filter: Option<String>) -> EventPattern {
        let detail = Detail::new(filter);

        EventPattern {
            detail_type: vec![event_name.to_string()],
            source: source,
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

        let bus = &config.aws.eventbridge.bus;
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
