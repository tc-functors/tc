use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "diffs/manifests.html")]
struct ManifestsTemplate {
    left: String,
    right: String
}

pub async fn generate() -> impl IntoResponse {

    let temp = ManifestsTemplate {
        left: String::from("a"),
        right: String::from("b")
    };
    Html(temp.render().unwrap())
}
