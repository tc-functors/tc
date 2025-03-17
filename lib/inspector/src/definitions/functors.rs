use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use std::collections::HashMap;
use compiler::{TopologyCount, Topology};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Functor {
    pub id: String,
    pub namespace: String,
    pub functions: usize,
    pub nodes: usize,
    pub events: usize,
    pub queues: usize,
    pub routes: usize,
    pub mutations: usize,
    pub states: usize,
    pub version: String
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Functor> {
    let mut xs: Vec<Functor> = vec![];
    for (_, topology) in &topologies {
        let t = TopologyCount::new(&topology);
        let f = Functor {
            id: topology.namespace.clone(),
            namespace: topology.namespace.clone(),
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
#[template(path = "definitions/fragments/functors.html")]
struct FunctorsTemplate {
    items: Vec<Functor>
}


pub async fn list() -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;
    let functors = build(topologies);
        let t = FunctorsTemplate {
            items: functors
        };
        Html(t.render().unwrap())
}

pub async fn compile() -> impl IntoResponse {
    let topologies = compiler::compile_root();
    cache::write("root", &serde_json::to_string(&topologies).unwrap()).await;
    let functors = build(topologies);
    let t = FunctorsTemplate {
        items: functors
    };
    Html(t.render().unwrap())
}
