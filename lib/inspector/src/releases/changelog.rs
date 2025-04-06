use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "releases/changelog.html")]
struct ChangelogTemplate {
    items: Vec<String>
}

pub async fn generate() -> impl IntoResponse {

    let temp = ChangelogTemplate {
        items: vec![]
    };
    Html(temp.render().unwrap())
}
