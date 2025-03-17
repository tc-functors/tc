use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use compiler::Topology;

#[derive(Template)]
#[template(path = "releases/fragments/changelog.html")]
struct ChangelogTemplate {
    items: Vec<String>
}

pub async fn list() -> impl IntoResponse {
    let t = ChangelogTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
