use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "sandboxes/list_hx.html")]
struct ListTemplate {
    items: Vec<String>
}

pub async fn generate() -> impl IntoResponse {
    let t = ListTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
