use crate::cache;
use askama::Template;
use axum::response::{
    Html,
    IntoResponse,
};
use composer::Topology;
use std::collections::HashMap;

struct Item {
    namespace: String,
    name: String,
    kind: String,
    target: String,
    input: String,
    output: String,
}

fn build_aux(topology: &Topology) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (_, mutation) in &topology.mutations {
        for (_, resolver) in &mutation.resolvers {
            let e = Item {
                namespace: topology.namespace.clone(),
                name: resolver.name.clone(),
                kind: resolver.entity.to_str(),
                target: resolver.target_arn.clone(),
                input: resolver.input.clone(),
                output: resolver.output.clone(),
            };
            xs.push(e);
        }
    }
    xs
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (_, topology) in topologies {
        let items = build_aux(&topology);
        xs.extend(items);
        for (_, node) in topology.nodes {
            let items = build_aux(&node);
            xs.extend(items)
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "overview/mutations.html")]
struct MutationsTemplate {
    items: Vec<Item>,
}

pub async fn list() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let temp = MutationsTemplate {
        items: build(topologies),
    };

    Html(temp.render().unwrap())
}
