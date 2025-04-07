use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "sandboxes/create.html")]
struct FormTemplate {
    items: Vec<String>
}

pub async fn form() -> impl IntoResponse {
    let t = FormTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
