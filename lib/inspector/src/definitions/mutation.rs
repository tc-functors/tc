use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::Topology;
use std::collections::HashMap;

struct Mutation {
    namespace: String,
    name: String,
    kind: String,
    target: String,
    input: String,
    output: String
}

fn build(topology: &Topology) -> Vec<Mutation> {
    let mut xs: Vec<Mutation> = vec![];

    for (_, mutation) in &topology.mutations {
        for (_, resolver) in &mutation.resolvers {
            let e = Mutation {
                namespace: topology.namespace.clone(),
                name: resolver.name.clone(),
                kind: resolver.kind.to_str(),
                target: resolver.target_arn.clone(),
                input: resolver.input.clone(),
                output: resolver.output.clone()

            };
        xs.push(e);
        }
    }
    xs
}

fn build_all(topologies: HashMap<String, Topology>) -> Vec<Mutation> {
    let mut xs: Vec<Mutation> = vec![];

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
#[template(path = "definitions/list/mutations.html")]
struct MutationsTemplate {
    items: Vec<Mutation>
 }

pub async fn list(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;

    if &id == "all" {

        let xs = build_all(topologies);
        let temp = MutationsTemplate {
            items: xs
        };
        Html(temp.render().unwrap())

    } else {
        let maybe_topology = topologies.get(&id);

        if let Some(t) = maybe_topology {
            tracing::debug!("Found topology");
            let temp = MutationsTemplate {
                items: build(&t)
            };
            Html(temp.render().unwrap())
        } else {
            let temp = MutationsTemplate {
                items: vec![]
            };
            Html(temp.render().unwrap())
        }
    }
}
