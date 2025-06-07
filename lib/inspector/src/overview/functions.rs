use crate::cache;
use askama::Template;
use axum::{
    response::{
        Html,
        IntoResponse,
    },
};
use compiler::{
    Function,
    Topology,
};
use std::collections::HashMap;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Item {
    namespace: String,
    name: String,
    package_type: String,
    dir: String,
    fqn: String,
    layers: Vec<String>,
    memory: i32,
    timeout: i32,
    runtime: String,
    role: String,
}

fn build_fns(namespace: &str, fns: HashMap<String, Function>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (dir, f) in fns {
        let fun = Item {
            namespace: namespace.to_string(),
            name: f.actual_name.clone(),
            dir: dir.to_string(),
            fqn: f.fqn.clone(),
            package_type: f.runtime.package_type.clone(),
            layers: f.runtime.layers.clone(),
            memory: f.runtime.memory_size.unwrap(),
            timeout: f.runtime.timeout.unwrap(),
            runtime: f.runtime.lang.to_str(),
            role: f.runtime.role.path,
        };
        xs.push(fun);
    }
    xs
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (_, topology) in topologies {
        let fns = build_fns(&topology.namespace, topology.functions);
        xs.extend(fns);
        for (_, node) in topology.nodes {
            let fns = build_fns(&node.namespace, node.functions);
            xs.extend(fns)
        }
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs.reverse();
    xs
}

#[derive(Template)]
#[template(path = "overview/functions.html")]
struct FunctionsTemplate {
    items: Vec<Item>,
}

pub async fn list() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let fns = build(topologies);
    let temp = FunctionsTemplate {
        items: fns,
    };
    Html(temp.render().unwrap())
}
