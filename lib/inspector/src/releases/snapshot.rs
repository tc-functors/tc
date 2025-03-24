use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "releases/view/snapshot.html")]
struct SnapshotTemplate {
    items: Vec<String>
}

pub async fn list() -> impl IntoResponse {
    let t = SnapshotTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
