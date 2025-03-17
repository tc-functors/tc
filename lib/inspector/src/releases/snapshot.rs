use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use compiler::Topology;

#[derive(Template)]
#[template(path = "releases/fragments/snapshot.html")]
struct SnapshotTemplate {
    items: Vec<String>
}

pub async fn list() -> impl IntoResponse {
    let t = SnapshotTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
