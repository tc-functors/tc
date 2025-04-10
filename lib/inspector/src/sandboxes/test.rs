use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "sandboxes/test_hx.html")]
struct TestTemplate {
    items: Vec<String>
}

pub async fn test() -> impl IntoResponse {
    let t = TestTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
