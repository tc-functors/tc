use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "sandboxes/list.html")]
struct FormTemplate {
    items: Vec<String>
}

pub async fn generate() -> impl IntoResponse {
    let t = FormTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
