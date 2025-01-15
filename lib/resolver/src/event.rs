use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Context, Topology};
use compiler::{Consumes, Mutations};
use aws::appsync;
use aws::Env;
use kit as u;
use kit::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Detail {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, Vec<String>>,
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

fn make_detail(filter: Option<String>) -> Option<Detail> {
    match filter {
        Some(f) => {
            let d: Detail = serde_json::from_str(&f).unwrap();
            Some(d)
        }
        None => None,
    }
}

fn make_pattern(event_name: &str, source: Vec<String>, filter: Option<String>) -> EventPattern {
    let detail = make_detail(filter);

    EventPattern {
        detail_type: vec![event_name.to_string()],
        source: source,
        detail: detail,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub arn: String,
    pub role_arn: String,
    pub input_paths_map: Option<HashMap<String, String>>,
    pub input_template: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub name: String,
    pub rule_name: String,
    pub bus: String,
    pub pattern: EventPattern,
    pub target: Target,
}

fn fqn_of(context: &Context, topology: &Topology, fn_name: &str) -> String {
    let Topology { functions, .. } = topology;
    for (_, f) in functions {
        if &f.name == fn_name {
            return context.render(&f.fqn);
        }
    }
    return context.render(fn_name);
}

fn determine_target_kind(function: &Option<String>, mutation: &Option<String>) -> String {
    if function.is_some() {
        s!("lambda")
    } else if mutation.is_some() {
        s!("appsync")
    } else {
        s!("sfn")
    }
}

// appsync targets
async fn get_graphql_arn_id(env: &Env, name: &str) -> Option<String> {
    let client = appsync::make_client(env).await;
    let api = appsync::find_api(&client, name).await;
    match api {
        Some(ap) => {
            let arn = appsync::get_api_endpoint(&client, &ap.id).await;
            match arn {
                Some(a) => {
                    let tmp = u::split_last(&a, "://");
                    Some(u::split_first(&tmp, "."))
                }
                None => None,
            }
        }
        None => None,
    }
}

async fn find_target_arn(env: &Env, target_kind: &str, target_name: &str, name: &str) -> String {
    match target_kind {
        "lambda" => env.lambda_arn(target_name),
        "sfn" => s!(target_name),
        "appsync" => {
            let id = get_graphql_arn_id(env, name).await;
            match id {
                Some(gid) => env.graphql_arn(&gid),
                None => s!(""),
            }
        }
        _ => env.sfn_arn(target_name),
    }
}

async fn find_mutation(mutation: &Option<String>, mutations: &Mutations) -> String {
    match mutation {
        Some(name) => {
            let Mutations {
                types_map,
                resolvers,
                ..
            } = mutations;
            let resolver = resolvers.get(name);
            let output = match resolver {
                Some(r) => &r.output,
                None => panic!("resolver output type not defined"),
            };

            let fields = types_map.get(output).expect("Not found").keys();
            let mut s: String = s!("");
            for f in fields {
                s.push_str(&format!(
                    r"{f}
"
                ))
            }
            let label = u::pascal_case(&name);
            format!(
                r#"mutation {label}($detail: String) {{
  {name}(detail: $detail) {{
    {s}
    createdAt
    updatedAt
  }}
}}"#
            )
        }
        None => s!(""),
    }
}

fn make_target(
    env: &Env,
    kind: &str,
    event_name: &str,
    target_name: &str,
    target_arn: &str,
    mutation: Option<String>,
) -> Target {
    let role_name = env.base_role("event");
    let role_arn = &env.role_arn(&role_name);

    let input_paths_map = match mutation {
        Some(_) => {
            let mut h: HashMap<String, String> = HashMap::new();
            h.insert(s!("detail"), s!("$.detail"));
            Some(h)
        }
        None => None,
    };

    let target_id = match kind {
        "lambda" | "mutation" | "appsync" => format!("{}_{}_target", event_name, kind),
        _ => format!("{}_target", event_name)
    };

    Target {
        kind: s!(kind),
        id: target_id,
        name: s!(target_name),
        arn: s!(target_arn),
        role_arn: s!(role_arn),
        input_paths_map: input_paths_map,
        input_template: Some(format!(r##"{{"detail": <detail>}}"##)),
    }
}

async fn find_target_name(
    kind: &str,
    context: &Context,
    topology: &Topology,
    function: &Option<String>,
    mutation: &Option<String>,
) -> String {
    let Context { env, name, .. } = context;
    let sfn_arn = &env.sfn_arn(name);

    match kind {
        "sfn" => s!(sfn_arn),
        "lambda" => {
            let fqn = match function {
                Some(f) => fqn_of(context, topology, &f),
                None => u::empty(),
            };
            fqn
        }
        "appsync" => find_mutation(mutation, &topology.mutations.clone().unwrap()).await,
        _ => s!(sfn_arn),
    }
}

async fn make_consumer(
    context: &Context,
    topology: &Topology,
    consumes: Consumes,
    event_name: &str,
) -> Option<Event> {
    let Consumes {
        producer,
        filter,
        function,
        mutation,
        pattern,
        sandboxes,
        ..
    } = consumes;
    let Context {
        env,
        name,
        namespace,
        sandbox,
        ..
    } = context;

    let rule_name = format!("tc-{}-{}-{}", namespace, event_name, &sandbox);
    let source = vec![context.render(&producer)];
    let pattern = match pattern {
        Some(p) => {
            let pp: EventPattern = serde_json::from_str(&p).unwrap();
            pp
        }
        None => make_pattern(event_name, source, filter),
    };

    let target_kind = determine_target_kind(&function, &mutation);
    let target_name = find_target_name(&target_kind, context, topology, &function, &mutation).await;
    let target_arn = find_target_arn(env, &target_kind, &target_name, &name).await;
    let target = make_target(
        env,
        &target_kind,
        event_name,
        &target_name,
        &target_arn,
        mutation,
    );

    if sandbox == "stable" || sandboxes.contains(&sandbox) {
        let event = Event {
            name: s!(event_name),
            rule_name: rule_name,
            bus: env.config.aws.eventbridge.bus.to_owned(),
            pattern: pattern,
            target: target,
        };
        Some(event)
    } else {
        None
    }
}

fn abbr(name: &str) -> String {
    if name.chars().count() > 15 {
        u::abbreviate(name, "-")
    } else {
        name.to_string()
    }
}

async fn make_mutation_consumer(context: &Context, topology: &Topology, consumes: Consumes, event_name: &str) -> Option<Event> {
    let Consumes { producer, filter, mutation, pattern, sandboxes, .. } = consumes;
    let Context { env, name, namespace, sandbox, .. } = context;

    let rule_name = format!("tc-{}-{}-{}-mutation", abbr(namespace), event_name, &sandbox);
    let source = vec![context.render(&producer)];
    let pattern = match pattern {
        Some(p) => {
            let pp: EventPattern = serde_json::from_str(&p).unwrap();
            pp
        },
        None => make_pattern(event_name, source, filter),
    };

    let function = None;
    let target_kind = determine_target_kind(&function, &mutation);
    let target_name = find_target_name(&target_kind, context, topology, &function, &mutation).await;
    let target_arn = find_target_arn(env, &target_kind, &target_name, &name).await;
    let target = make_target(env, &target_kind, event_name, &target_name, &target_arn, mutation);

    if sandbox == "stable" || sandboxes.contains(&sandbox) {
        let event = Event {
            name: s!(event_name),
            rule_name: rule_name,
            bus: env.config.aws.eventbridge.bus.clone(),
            pattern: pattern,
            target: target
        };
        Some(event)
    } else {
        None
    }
}

async fn make_function_consumer(context: &Context, topology: &Topology, consumes: Consumes, event_name: &str) -> Option<Event> {
    let Consumes { producer, filter, function, pattern, sandboxes, .. } = consumes;
    let Context { env,  name, namespace, sandbox, .. } = context;

    let rule_name = format!("tc-{}-{}-{}-fn", abbr(namespace), event_name, &sandbox);
    let source = vec![context.render(&producer)];
    let pattern = match pattern {
        Some(p) => {
            let pp: EventPattern = serde_json::from_str(&p).unwrap();
            pp
        },
        None => make_pattern(event_name, source, filter),
    };

    let mutation = None;
    let target_kind = determine_target_kind(&function, &mutation);
    let target_name = find_target_name(&target_kind, context, topology, &function, &mutation).await;
    let target_arn = find_target_arn(env, &target_kind, &target_name, &name).await;
    let target = make_target(env, &target_kind, event_name, &target_name, &target_arn, mutation);

    if sandbox == "stable" || sandboxes.contains(&sandbox) {
        let event = Event {
            name: s!(event_name),
            rule_name: rule_name,
            bus: env.config.aws.eventbridge.bus.clone(),
            pattern: pattern,
            target: target
        };
        Some(event)
    } else {
        None
    }
}


pub async fn make(context: &Context, topology: &Topology) -> Vec<Event> {
    let mut events: Vec<Event> = vec![];

    if let Some(e) = &topology.events {
        let consumes = match e.consumes.clone() {
            Some(c) => c,
            None => HashMap::new(),
        };

        for (event_name, event_spec) in consumes {
            if let Some(_) = &event_spec.mutation {
                let event = make_mutation_consumer(context, topology, event_spec.clone(), &event_name).await;
                match event {
                    Some(e) => events.push(e),
                    None => (),
                }
            }

            if let Some(_) = &event_spec.function {
                let event = make_function_consumer(context, topology, event_spec.clone(), &event_name).await;
                match event {
                    Some(e) => events.push(e),
                    None => (),
                }
            }

            if !kit::option_exists(event_spec.mutation.clone()) && !kit::option_exists(event_spec.function.clone()) {

                let event = make_consumer(context, topology, event_spec, &event_name).await;
                match event {
                    Some(e) => events.push(e),
                    None => (),
                }
            }
        }
    }

    events
}
