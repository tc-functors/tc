use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "sandboxes/list.html")]
struct FunctorsTemplate {
    items: Vec<String>
}

pub async fn list() -> impl IntoResponse {
    let t = FunctorsTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
