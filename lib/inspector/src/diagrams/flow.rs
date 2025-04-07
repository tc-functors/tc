use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "diagrams/flow.html")]
struct FlowTemplate {
    items: Vec<String>
}

pub async fn generate() -> impl IntoResponse {

    let temp = FlowTemplate {
        items: vec![]
    };
    Html(temp.render().unwrap())
}
