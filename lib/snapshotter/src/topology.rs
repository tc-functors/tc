use crate::aws::{
    appsync,
    eventbridge,
    lambda,
};
use authorizer::Auth;
use kit as u;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Mutation {
    id: String,
    https: String,
    wss: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct States {
    mode: String,
    definition: Value,
}

async fn find_states(_auth: &Auth, _fqn: &str) -> Option<States> {
    None
}

async fn find_routes(_auth: &Auth, _fqn: &str) -> HashMap<String, String> {
    HashMap::new()
}

async fn find_mutations(auth: &Auth, fqn: &str) -> Option<Mutation> {
    let client = appsync::make_client(auth).await;
    let api = appsync::find_api(&client, fqn).await;
    match api {
        Some(a) => {
            let m = Mutation {
                id: a.id.clone(),
                https: a.https.clone(),
                wss: a.wss,
            };
            Some(m)
        }
        _ => None,
    }
}

// events
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub rule: String,
    pub event: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Pattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "detail-type")]
    detail_type: Option<Vec<String>>,
}

pub async fn find_events(auth: &Auth, namespace: &str) -> HashMap<String, Event> {
    let client = eventbridge::make_client(auth).await;
    // fixme
    let bus = String::from("default");
    let rules = eventbridge::list_rules(client.clone(), bus.clone(), namespace.to_string()).await;

    let mut evs: HashMap<String, Event> = HashMap::new();
    for rule in rules {
        let p = rule.event_pattern.unwrap();
        let pattern: Pattern = serde_json::from_str(&p).unwrap();
        let rule_name = rule.name.unwrap();
        let target = eventbridge::get_target(client.clone(), bus.clone(), rule_name.clone()).await;
        let ev = Event {
            rule: rule_name,
            event: u::maybe_vec_string(pattern.detail_type.clone()),
            target: u::split_last(&target, ":"),
        };
        evs.insert(u::maybe_vec_string(pattern.detail_type), ev);
    }
    evs
}

// functions

#[derive(Serialize, Deserialize, Debug)]
struct Function {
    name: String,
    code_size: String,
    timeout: i32,
    mem: i32,
    revision: String,
    updated: String,
    layers: HashMap<String, i64>,
    tc_version: String,
}

async fn find_functions(auth: &Auth, fns: Vec<String>) -> HashMap<String, Function> {
    let client = lambda::make_client(auth).await;
    let mut h: HashMap<String, Function> = HashMap::new();
    for f in fns {
        let tags = lambda::list_tags(client.clone(), &auth.lambda_arn(&f))
            .await
            .unwrap();

        let config = lambda::find_config(&client, &auth.lambda_arn(&f)).await;

        match config {
            Some(cfg) => {
                let layers = lambda::find_function_layers(&client, &f).await.unwrap();
                let row = Function {
                    name: f.clone(),
                    code_size: u::file_size_human(cfg.code_size as f64),
                    timeout: cfg.timeout,
                    mem: cfg.mem_size,
                    revision: cfg.revision,
                    tc_version: u::safe_unwrap(tags.get("tc_version")),
                    updated: u::safe_unwrap(tags.get("updated_at")),
                    layers: layers,
                };
                h.insert(f, row);
            }
            None => (),
        }
    }
    h
}

pub fn render(s: &str, namespace: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    table.insert("namespace", namespace);
    u::stencil(s, table)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Topology {
    namespace: String,
    functions: HashMap<String, Function>,
    events: HashMap<String, Event>,
    routes: HashMap<String, String>,
    mutations: Option<Mutation>,
    states: Option<States>,
}

impl Topology {
    pub async fn new(auth: &Auth, dir: &str, sandbox: &str) -> Topology {
        let t = compiler::compile(dir, true);
        let mut fns: Vec<String> = vec![];
        for (_, f) in t.functions {
            let name = render(&f.fqn, &t.namespace, sandbox);
            fns.push(name);
        }

        Topology {
            namespace: t.namespace.clone(),
            functions: find_functions(auth, fns).await,
            events: find_events(auth, &t.namespace).await,
            mutations: find_mutations(auth, &t.namespace).await,
            routes: find_routes(auth, &t.namespace).await,
            states: find_states(auth, &t.fqn).await,
        }
    }
}
