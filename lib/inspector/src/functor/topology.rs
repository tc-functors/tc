use crate::Store;
use askama::Template;
use axum::{
    extract::{
        Path,
        State,
    },
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

pub async fn test(
    State(_store): State<Store>,
    Path((_root, _namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let temp = FormTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functor/topology/compose.html")]
struct ViewTemplate {
    definition: String,
}

pub async fn compose(
    State(store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let f = store.find_topology(&root, &namespace).await;
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

pub async fn definition(
    State(store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let f = store.find_topology(&root, &namespace).await;
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
