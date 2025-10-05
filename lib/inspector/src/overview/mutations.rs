use crate::Store;
use askama::Template;
use axum::{
    extract::State,
    response::{
        Html,
        IntoResponse,
    },
};
use composer::Topology;

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

fn build(topologies: Vec<Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for topology in topologies {
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

pub async fn list(State(store): State<Store>) -> impl IntoResponse {
    let topologies = store.list_topologies().await;
    let temp = MutationsTemplate {
        items: build(topologies),
    };

    Html(temp.render().unwrap())
}
