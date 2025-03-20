use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::{TopologyCount, Topology};
use std::collections::HashMap;


struct Node {
    parent: String,
    namespace: String,
    functions: usize,
    events: usize,
    queues: usize,
    routes: usize,
    mutations: usize,
    version: String
}

fn build_node(parent: &str, t: &Topology) -> Node {
    let tc = TopologyCount::new(&t);
    Node {
        parent: String::from(parent),
        namespace: t.namespace.clone(),
        functions: tc.functions,
        events: tc.events,
        queues: tc.queues,
        routes: tc.routes,
        mutations: tc.mutations,
        version: String::from(&t.version),
    }
}

fn build(parent: &str, root: &Topology) -> Vec<Node> {
    let mut xs: Vec<Node> = vec![];
    for (_, node) in &root.nodes {
        let f = build_node(parent, &node);
        xs.push(f)
    }
    xs
}

fn build_all(topologies: HashMap<String, Topology>) -> Vec<Node> {

    let mut xs: Vec<Node> = vec![];

    for (_, topology) in topologies {

        let ns = build_node(&topology.namespace, &topology);
        xs.push(ns);

        for (_, node) in topology.nodes {
            let fns = build(&topology.namespace, &node);
            xs.extend(fns)
        }
    }
    xs.sort_by(|a, b| b.parent.cmp(&a.parent));
    xs.reverse();
    xs
}

#[derive(Template)]
#[template(path = "definitions/list/nodes.html")]
struct NodesTemplate {
    id: String,
    items: Vec<Node>
 }

pub async fn list(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;

   if &id == "all" {

        let nodes = build_all(topologies);
        let temp = NodesTemplate {
            id: String::from("all"),
            items: nodes
        };
        Html(temp.render().unwrap())

    } else {

    let maybe_topology = topologies.get(&id);

    if let Some(t) = maybe_topology {
        tracing::debug!("Found topology");
        let temp = NodesTemplate {
            id: id,
            items: build(&t.namespace, &t)
        };
        Html(temp.render().unwrap())
    } else {
        let temp = NodesTemplate {
            id: id,
            items: vec![]
        };
        Html(temp.render().unwrap())
    }
   }
}

#[derive(Template)]
#[template(path = "definitions/view/node.html")]
struct ViewTemplate {
    item: String
}

pub async fn view(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;
    let f = topologies.get(&id);
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
