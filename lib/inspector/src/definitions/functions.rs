use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use compiler::Topology;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Function {
    name: String,
    dir: String,
    fqn: String,
    layers: Vec<String>,
    memory: i32,
    timeout: i32,
    runtime: String,
}

#[derive(Template)]
#[template(path = "definitions/fragments/functions.html")]
struct FunctionsTemplate {
    items: Vec<Function>
 }

fn build(t: &Topology) -> Vec<Function> {
    let mut xs: Vec<Function> = vec![];

    for (dir, f) in &t.functions() {
        let fun = Function {
            name: f.actual_name.clone(),
            dir: dir.to_string(),
            fqn: f.fqn.clone(),
            layers: f.runtime.layers.clone(),
            memory: f.runtime.memory_size.unwrap(),
            timeout: f.runtime.timeout.unwrap(),
            runtime: f.runtime.lang.to_str()
        };
        xs.push(fun);
    }
    xs.sort_by(|a, b| b.name.cmp(&a.name));
    xs
}

pub async fn list(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;
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
