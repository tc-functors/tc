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
    Page,
    Topology,
};

use std::collections::HashMap;

struct Item  {
    namespace: String,
    name: String,
    bucket: String,
}

#[derive(Template)]
#[template(path = "overview/pages.html")]
struct PagesTemplate {
    items: Vec<Item>,
}

fn build_pages(namespace: &str, rs: HashMap<String, Page>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (name, page) in rs {
        let e = Item {
            namespace: namespace.to_string(),
            name: name,
            bucket: page.bucket,
        };
        xs.push(e);
    }
    xs
}

fn build(topologies: Vec<Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for topology in topologies {
        let rs = build_pages(&topology.namespace, topology.pages);
        xs.extend(rs);
        for (_, node) in topology.nodes {
            let rs = build_pages(&node.namespace, node.pages);
            xs.extend(rs)
        }
    }
    xs
}

pub async fn list(State(store): State<Store>) -> impl IntoResponse {
    let topologies = store.list_topologies().await;
    let temp = PagesTemplate { items: build(topologies) };
    Html(temp.render().unwrap())
}
