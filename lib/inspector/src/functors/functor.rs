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
#[template(path = "functors/functor/flow.html")]
struct FlowTemplate {}

pub async fn flow(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = FlowTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functors/functor/sandbox_form.html")]
struct SandboxTemplate {}

pub async fn sandbox(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = SandboxTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functors/functor/test_form.html")]
struct FormTemplate {}

pub async fn test(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = FormTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functors/functor/topology.html")]
struct ViewTemplate {
    topology: String,
}

pub async fn compile(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&root, &namespace).await;
    if let Some(t) = f {
        let temp = ViewTemplate {
            topology: t.to_str(),
        };
        Html(temp.render().unwrap())
    } else {
        let temp = ViewTemplate {
            topology: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}
