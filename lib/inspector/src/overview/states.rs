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
    Topology,
};

struct Item {
    namespace: String,
    mode: String,
    role: String,
}


fn build(topologies: Vec<Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for topology in topologies {
        if let Some(flow) = topology.flow {
            let item = Item {
                namespace: flow.name,
                mode: flow.mode,
                role: flow.role.name,
            };
            xs.push(item);
        }

        for (_, node) in topology.nodes {
            if let Some(flow) = node.flow {
                let item = Item {
                    namespace: flow.name,
                    mode: flow.mode,
                    role: flow.role.name
                };
                xs.push(item);
            }
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "overview/states.html")]
struct StatesTemplate {
    items: Vec<Item>,
}

pub async fn list(State(store): State<Store>) -> impl IntoResponse {
    let topologies = store.list_topologies().await;
    let states = build(topologies);
    let temp = StatesTemplate { items: states };
    Html(temp.render().unwrap())
}
