use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use compiler::Topology;
use std::collections::HashMap;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Functor {
    pub id: String,
    pub namespace: String,
    pub version: String,
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Functor> {
    let mut xs: Vec<Functor> = vec![];
    for (name, topology) in &topologies {
        let f = Functor {
            id: topology.namespace.clone(),
            namespace: topology.namespace.clone(),
            version: String::from(&topology.version),
        };
        xs.push(f)
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs.reverse();
    xs
}

#[derive(Template)]
#[template(path = "releases/fragments/functors.html")]
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
