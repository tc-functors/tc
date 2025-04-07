use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "diagrams/c4.html")]
struct C4Template {
    items: Vec<String>
}

pub async fn generate() -> impl IntoResponse {

    let temp = C4Template {
        items: vec![]
    };
    Html(temp.render().unwrap())
}
