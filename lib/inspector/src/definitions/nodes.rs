use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::{TopologyCount, Topology};
use crate::definitions::functors::Functor;

#[derive(Template)]
#[template(path = "definitions/fragments/nodes.html")]
struct NodesTemplate {
    id: String,
    items: Vec<Functor>
 }

async fn build(root: &Topology) -> Vec<Functor> {
    let mut xs: Vec<Functor> = vec![];
    tracing::debug!("{}", &root.nodes.len());
    for node in &root.nodes {
        let t = TopologyCount::new(&node);
        let f = Functor {
            id: node.fqn.clone(),
            namespace: node.namespace.clone(),
            functions: t.functions,
            nodes: t.nodes,
            events: t.events,
            queues: t.queues,
            routes: t.routes,
            states: t.states,
            mutations: t.mutations,
            version: String::from(&node.version),
        };
        xs.push(f)
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs.reverse();
    xs
}

pub async fn list(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;
    let maybe_topology = topologies.get(&id);

    if let Some(t) = maybe_topology {
        tracing::debug!("Found topology");
        let temp = NodesTemplate {
            id: id,
            items: build(&t).await
        };
        Html(temp.render().unwrap())
    } else {
        let temp = NodesTemplate {
            id: id,
            items: vec![]
        };
        Html(temp.render().unwrap())
    }
}
