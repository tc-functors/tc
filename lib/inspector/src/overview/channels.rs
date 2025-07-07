use crate::cache;
use askama::Template;
use axum::response::{
    Html,
    IntoResponse,
};
use composer::{
    Event,
    Topology,
};
use std::collections::HashMap;

struct Item {
    namespace: String,
    name: String,
    rule_name: String,
    pattern: String,
    targets: HashMap<String, String>,
}

fn build_events(namespace: &str, evs: HashMap<String, Event>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (_, event) in evs {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &event.targets {
            targets.insert(t.entity.to_str(), t.name.clone());
        }
        let e = Item {
            namespace: namespace.to_string(),
            name: event.name.clone(),
            rule_name: event.rule_name.clone(),
            pattern: serde_json::to_string(&event.pattern).unwrap(),
            targets: targets,
        };
        xs.push(e);
    }
    xs
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (_, topology) in topologies {
        let fns = build_events(&topology.namespace, topology.events);
        xs.extend(fns);
        for (_, node) in topology.nodes {
            let fns = build_events(&node.namespace, node.events);
            xs.extend(fns)
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "overview/channels.html")]
struct EventsTemplate {
    items: Vec<Item>,
}

pub async fn list() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let events = build(topologies);
    let temp = EventsTemplate { items: events };
    Html(temp.render().unwrap())
}
