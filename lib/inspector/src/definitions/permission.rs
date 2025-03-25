use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use compiler::{Function, Topology};
use std::collections::HashMap;
use crate::store;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Item {
    namespace: String,
    name: String,
    dir: String,
    role: String,
    vars: String
}

fn build_fns(namespace: &str, fns: HashMap<String, Function>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (dir, f) in fns {
        let fun = Item {
            namespace: namespace.to_string(),
            name: f.actual_name.clone(),
            dir: dir.to_string(),
            role: f.runtime.role.path,
            vars: String::from("")
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
#[template(path = "definitions/list/permissions.html")]
struct PermissionsTemplate {
    functions: Vec<Item>
}

pub async fn list_all() -> impl IntoResponse {
    let topologies = store::find_all_topologies().await;
    let fns = build(topologies);
    let temp = PermissionsTemplate {
        functions: fns
    };
    Html(temp.render().unwrap())
}
