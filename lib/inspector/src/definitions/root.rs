use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use std::collections::HashMap;
use compiler::{TopologyCount, Topology};
use crate::cache;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Functor {
    root: String,
    namespace: String,
    kind: String,
    functions: usize,
    nodes: usize,
    events: usize,
    queues: usize,
    routes: usize,
    mutations: usize,
    states: usize,
    version: String
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Functor> {
    let mut xs: Vec<Functor> = vec![];
    for (_, topology) in &topologies {
        let t = TopologyCount::new(&topology);
        let f = Functor {
            root: topology.namespace.clone(),
            namespace: topology.namespace.clone(),
            kind: t.kind,
            functions: t.functions,
            nodes: t.nodes,
            events: t.events,
            queues: t.queues,
            routes: t.routes,
            states: t.states,
            mutations: t.mutations,
            version: String::from(&topology.version),
        };
        xs.push(f)
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs.reverse();
    xs
}

#[derive(Template)]
#[template(path = "definitions/list/root.html")]
struct FunctorsTemplate {
    items: Vec<Functor>
}


pub async fn list_all() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let functors = build(topologies);
        let t = FunctorsTemplate {
            items: functors
        };
        Html(t.render().unwrap())
}

pub async fn compile() -> impl IntoResponse {
    let topologies = compiler::compile_root(&kit::pwd(), true);
    cache::write("root", &serde_json::to_string(&topologies).unwrap()).await;
    let functors = build(topologies);
    let t = FunctorsTemplate {
        items: functors
    };
    Html(t.render().unwrap())
}



#[derive(Template)]
#[template(path = "definitions/view/graph.html")]
struct GraphTemplate {
    item: String
}

pub async fn generate_graph() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let mut h: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    for (name, node) in topologies {
        let subg = node.names_of();
        h.insert(name, subg);
    }
    let graph_str = serde_json::to_string(&h).unwrap();
    let t = GraphTemplate {
        item: graph_str
    };
    Html(t.render().unwrap())
}
