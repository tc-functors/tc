use crate::cache;
use askama::Template;
use axum::{
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
};
use compiler::{
    Route,
    Topology,
};
use std::collections::HashMap;

struct Item {
    namespace: String,
    method: String,
    path: String,
    gateway: String,
    authorizer: String,
    target_kind: String,
    target_arn: String,
}

fn build_routes(namespace: &str, rs: HashMap<String, Route>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (_, route) in rs {
        let e = Item {
            namespace: namespace.to_string(),
            method: route.method.clone(),
            path: route.path.clone(),
            gateway: route.gateway.clone(),
            authorizer: route.authorizer.clone(),
            target_kind: route.entity.to_str(),
            target_arn: route.target_arn.clone(),
        };
        xs.push(e);
    }
    xs
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (_, topology) in topologies {
        let rs = build_routes(&topology.namespace, topology.routes);
        xs.extend(rs);
        for (_, node) in topology.nodes {
            let rs = build_routes(&node.namespace, node.routes);
            xs.extend(rs)
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "overview/list/routes.html")]
struct RoutesTemplate {
    items: Vec<Item>,
}

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let routes = cache::find_routes(&root, &namespace).await;
    let temp = RoutesTemplate {
        items: build_routes(&namespace, routes),
    };
    Html(temp.render().unwrap())
}

pub async fn list_all() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let temp = RoutesTemplate {
        items: build(topologies),
    };
    Html(temp.render().unwrap())
}
