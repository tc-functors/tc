use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "releases/list/changelog.html")]
struct ChangelogTemplate {
    items: Vec<String>
}

pub async fn list() -> impl IntoResponse {
    let temp = ChangelogTemplate {
        items: vec![]
    };
    Html(temp.render().unwrap())
}
