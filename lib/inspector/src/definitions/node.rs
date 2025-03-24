use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::{TopologyCount, Topology};
use std::collections::HashMap;
use crate::store;

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
#[template(path = "definitions/list/nodes.html")]
struct NodesTemplate {
    items: Vec<Item>
 }

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let topologies = store::find_topologies(&root, &namespace).await;
    let nodes = build_nodes(&namespace, topologies);
    let temp = NodesTemplate {
        items: nodes
    };
    Html(temp.render().unwrap())
}

pub async fn list_all() -> impl IntoResponse {
    let topologies = store::find_all_topologies().await;
    let nodes = build(topologies);
    let temp = NodesTemplate {
        items: vec![]
    };
    Html(temp.render().unwrap())
}


// view


#[derive(Template)]
#[template(path = "definitions/view/node.html")]
struct ViewTemplate {
    item: String
}

pub async fn view(Path((root, namespace, _id)): Path<(String, String, String)>) -> impl IntoResponse {
    let f = store::find_topology(&root, &namespace).await;
    if let Some(t) = f {
        let temp = ViewTemplate {
            item: t.to_str()
        };
        Html(temp.render().unwrap())

    } else {
        let temp = ViewTemplate {
            item: String::from("test")
        };
        Html(temp.render().unwrap())
    }
}
