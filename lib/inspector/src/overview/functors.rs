use crate::cache;
use askama::Template;
use axum::response::{
    Html,
    IntoResponse,
};
use compiler::{
    Topology,
    TopologyCount,
};
use std::collections::HashMap;

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
    version: String,
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
#[template(path = "overview/functors.html")]
struct FunctorsTemplate {
    items: Vec<Functor>,
}

pub async fn list() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let functors = build(topologies);
    let t = FunctorsTemplate { items: functors };
    Html(t.render().unwrap())
}
