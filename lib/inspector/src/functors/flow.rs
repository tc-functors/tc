use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "functors/flow.html")]
struct FlowTemplate {
}

pub async fn view(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {

    let temp = FlowTemplate {
    };
    Html(temp.render().unwrap())
}
