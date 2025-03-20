use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::Topology;
use std::collections::HashMap;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Function {
    namespace: String,
    name: String,
    package_type: String,
    dir: String,
    fqn: String,
    layers: Vec<String>,
    memory: i32,
    timeout: i32,
    runtime: String,
}

#[derive(Template)]
#[template(path = "definitions/list/functions.html")]
struct FunctionsTemplate {
    items: Vec<Function>
}

fn build(t: &Topology) -> Vec<Function> {
    let mut xs: Vec<Function> = vec![];

    for (dir, f) in &t.functions() {
        let fun = Function {
            namespace: t.namespace.clone(),
            name: f.actual_name.clone(),
            dir: dir.to_string(),
            fqn: f.fqn.clone(),
            package_type: f.runtime.package_type.clone(),
            layers: f.runtime.layers.clone(),
            memory: f.runtime.memory_size.unwrap(),
            timeout: f.runtime.timeout.unwrap(),
            runtime: f.runtime.lang.to_str()
        };
        xs.push(fun);
    }
    xs
}

fn build_all(topologies: HashMap<String, Topology>) -> Vec<Function> {

    let mut xs: Vec<Function> = vec![];

    for (_, topology) in topologies {
        for (_, node) in topology.nodes {
            let fns = build(&node);
            xs.extend(fns)
        }
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs
}

pub async fn list(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;

    if &id == "all" {

        let fns = build_all(topologies);
        let temp = FunctionsTemplate {
            items: fns
        };
        Html(temp.render().unwrap())

    } else {
        let maybe_topology = topologies.get(&id);

        if let Some(t) = maybe_topology {
            tracing::debug!("Found topology");
            let temp = FunctionsTemplate {
                items: build(&t)
            };
            Html(temp.render().unwrap())
        } else {
            let temp = FunctionsTemplate {
                items: vec![]
            };
            Html(temp.render().unwrap())
        }
    }
}


#[derive(Template)]
#[template(path = "definitions/view/function.html")]
struct ViewTemplate {
    item: String
}

pub async fn view(Path(id): Path<String>) -> impl IntoResponse {
    let temp = ViewTemplate {
        item: String::from("test")
    };
    Html(temp.render().unwrap())
}
