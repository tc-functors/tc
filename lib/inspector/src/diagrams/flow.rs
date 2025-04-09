use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "diagrams/flow.html")]
struct FlowTemplate {
}

pub async fn generate() -> impl IntoResponse {

    let temp = FlowTemplate {
    };
    Html(temp.render().unwrap())
}
