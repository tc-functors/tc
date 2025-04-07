use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "releases/timeline.html")]
struct ViewTemplate {
    entity: String,
    context: String,
    items: Vec<String>
}

pub async fn view() -> impl IntoResponse {
    let t = ViewTemplate {
        entity: String::from("timeline"),
        context: String::from("releases"),
        items: vec![]
    };
    Html(t.render().unwrap())
}
