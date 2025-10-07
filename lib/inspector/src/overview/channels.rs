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
    Channel,
    Topology,
};
use std::collections::HashMap;

struct Item {
    name: String,
    targets: HashMap<String, String>,
}

fn build_channels(_namespace: &str, channels: HashMap<String, Channel>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (_, channel) in channels {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &channel.targets {
            targets.insert(t.entity.to_str(), t.name.clone());
        }
        let e = Item {
            name: channel.name.to_string(),
            targets: targets,
        };
        xs.push(e);
    }
    xs
}

fn build(topologies: Vec<Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for topology in topologies {
        let fns = build_channels(&topology.namespace, topology.channels);
        xs.extend(fns);
        for (_, node) in topology.nodes {
            let fns = build_channels(&node.namespace, node.channels);
            xs.extend(fns)
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "overview/channels.html")]
struct ChannelsTemplate {
    items: Vec<Item>,
}

pub async fn list(State(store): State<Store>) -> impl IntoResponse {
    let topologies = store.list_topologies().await;
    let channels = build(topologies);
    let temp = ChannelsTemplate { items: channels };
    Html(temp.render().unwrap())
}
