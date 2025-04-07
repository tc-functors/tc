use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "diffs/functors.html")]
struct FunctorsTemplate {
    entity: String,
    context: String,
    left: String,
    right: String
}

pub async fn view() -> impl IntoResponse {
    let temp = FunctorsTemplate {
        entity: String::from("functors"),
        context: String::from("diffs"),
        left: String::from("a"),
        right: String::from("b")
    };
    Html(temp.render().unwrap())
}
