use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::Topology;
use std::collections::HashMap;

struct Route {
    namespace: String,
    method: String,
    path: String,
    gateway: String,
    authorizer: String,
    target_kind: String,
    target_arn: String,
}


fn build(topology: &Topology) -> Vec<Route> {
    let mut xs: Vec<Route> = vec![];

    for (_, route) in &topology.routes {
        let e = Route {
            namespace: topology.namespace.clone(),
            method: route.method.clone(),
            path: route.path.clone(),
            gateway: route.gateway.clone(),
            authorizer: route.authorizer.clone(),
            target_kind: route.target_kind.to_str(),
            target_arn: route.target_arn.clone()

        };
        xs.push(e);
    }
    xs
}

fn build_all(topologies: HashMap<String, Topology>) -> Vec<Route> {
    let mut xs: Vec<Route> = vec![];

    for (_, topology) in topologies {
        let fns = build(&topology);
        xs.extend(fns);
        for (_, node) in topology.nodes {
            let fns = build(&node);
            xs.extend(fns)
        }
    }
    xs
}


#[derive(Template)]
#[template(path = "definitions/list/routes.html")]
struct RoutesTemplate {
    items: Vec<Route>
 }

pub async fn list(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;

    if &id == "all" {

        let xs = build_all(topologies);
        let temp = RoutesTemplate {
            items: xs
        };
        Html(temp.render().unwrap())

    } else {
        let maybe_topology = topologies.get(&id);

        if let Some(t) = maybe_topology {
            tracing::debug!("Found topology");
            let temp = RoutesTemplate {
                items: build(&t)
            };
            Html(temp.render().unwrap())
        } else {
            let temp = RoutesTemplate {
                items: vec![]
            };
            Html(temp.render().unwrap())
        }
    }
}
