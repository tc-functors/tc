use crate::cache;
use askama::Template;
use axum::{
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
};

#[derive(Template)]
#[template(path = "functor/topology/flow.html")]
struct FlowTemplate {}

pub async fn flow(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = FlowTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functor/topology/sandbox_form.html")]
struct SandboxTemplate {}

pub async fn sandbox(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = SandboxTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functor/topology/test_form.html")]
struct FormTemplate {}

pub async fn test(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = FormTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functor/topology/compile.html")]
struct ViewTemplate {
    definition: String,
}

pub async fn compile(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&root, &namespace).await;
    if let Some(t) = f {
        let temp = ViewTemplate {
            definition: t.to_str(),
        };
        Html(temp.render().unwrap())
    } else {
        let temp = ViewTemplate {
            definition: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}


#[derive(Template)]
#[template(path = "functor/topology.html")]
struct DefinitionTemplate {
    root: String,
    namespace: String,
    definition: String,
}

fn lookup_definition(dir: &str) -> String {
    let f = format!("{}/topology.yml", dir);
    kit::slurp(&f)
}

pub async fn definition(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&root, &namespace).await;
    if let Some(t) = f {
        let definition = lookup_definition(&t.dir);
        let temp = DefinitionTemplate {
            root: root,
            namespace: namespace,
            definition: definition,
        };
        Html(temp.render().unwrap())
    } else {
        let temp = DefinitionTemplate {
            root: root,
            namespace: namespace,
            definition: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}


pub struct TopologyCount {
    pub functions: usize,
    pub events: usize,
    pub routes: usize,
    pub mutations: usize,
    pub queues: usize,
    pub channels: usize,
    pub states: usize
}

pub async fn count_of(root: &str, namespace: &str) -> TopologyCount {
    let f = cache::find_topology(root, namespace).await;
    if let Some(t) = f {

        TopologyCount {
            functions: t.functions.len(),
            events: t.events.len(),
            routes: t.routes.len(),
            mutations: t.mutations.len(),
            queues: t.queues.len(),
            channels: t.channels.len(),
            states: 0
        }
    } else {
        TopologyCount {
            functions: 0,
            events: 0,
            routes: 0,
            mutations: 0,
            queues: 0,
            channels: 0,
            states: 0
        }
    }
}

pub fn name_of() -> String {
    compiler::topology_name(&kit::pwd())
}
