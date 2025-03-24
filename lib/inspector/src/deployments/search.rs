use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};


#[derive(Template)]
#[template(path = "deployments/view/search.html")]
struct SearchTemplate {
    items: Vec<String>
}

pub async fn list() -> impl IntoResponse {
    let t = SearchTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}
