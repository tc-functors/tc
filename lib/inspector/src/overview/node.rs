use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::{TopologyCount, Topology};
use std::collections::HashMap;
use crate::cache;

struct Item {
    root: String,
    namespace: String,
    functions: usize,
    events: usize,
    nodes: usize,
    queues: usize,
    routes: usize,
    mutations: usize,
    version: String
}


fn build_nodes(root: &str, nodes: HashMap<String, Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (_, node) in nodes {
        tracing::debug!("n: {}", &node.namespace);
        let tc = TopologyCount::new(&node);
        let item = Item {
            root: String::from(root),
            namespace: node.namespace.clone(),
            functions: tc.functions,
            nodes: tc.nodes,
            events: tc.events,
            queues: tc.queues,
            routes: tc.routes,
            mutations: tc.mutations,
            version: String::from(&node.version),
        };
        xs.push(item)
    }
    xs
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Item> {

    let mut xs: Vec<Item> = vec![];
    for (_, topology) in topologies {

        let ns = build_nodes(&topology.namespace, topology.nodes);
        xs.extend(ns)

    }
    xs.sort_by(|a, b| b.root.cmp(&a.root));
    xs.reverse();
    xs
}


#[derive(Template)]
#[template(path = "overview/list/nodes.html")]
struct NodesTemplate {
    items: Vec<Item>
 }

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let topologies = cache::find_topologies(&root, &namespace).await;
    let nodes = build_nodes(&namespace, topologies);
    let temp = NodesTemplate {
        items: nodes
    };
    Html(temp.render().unwrap())
}

pub async fn list_all() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let nodes = build(topologies);
    let temp = NodesTemplate {
        items: nodes
    };
    Html(temp.render().unwrap())
}
