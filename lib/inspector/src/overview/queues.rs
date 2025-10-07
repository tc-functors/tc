use crate::Store;
use askama::Template;
use axum::{
    extract::State,
    response::{
        Html,
        IntoResponse,
    },
};
use composer::{
    Queue,
    Topology,
};
use std::collections::HashMap;

struct Item {
    name: String,
    targets: HashMap<String, String>,
}

fn build_queues(_namespace: &str, queues: HashMap<String, Queue>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (_, queue) in queues {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &queue.targets {
            targets.insert(t.entity.to_str(), t.name.clone());
        }
        let e = Item {
            name: queue.name.to_string(),
            targets: targets
        };
        xs.push(e);
    }
    xs
}

fn build(topologies: Vec<Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for topology in topologies {
        let fns = build_queues(&topology.namespace, topology.queues);
        xs.extend(fns);
        for (_, node) in topology.nodes {
            let fns = build_queues(&node.namespace, node.queues);
            xs.extend(fns)
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "overview/queues.html")]
struct QueuesTemplate {
    items: Vec<Item>,
}

pub async fn list(State(store): State<Store>) -> impl IntoResponse {
    let topologies = store.list_topologies().await;
    let events = build(topologies);
    let temp = QueuesTemplate { items: events };
    Html(temp.render().unwrap())
}
