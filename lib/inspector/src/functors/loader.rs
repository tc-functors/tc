use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use crate::cache;

#[derive(Template)]
#[template(path = "functors/list.html")]
struct ListTemplate {
    namespace: String,
    items: Vec<String>
}

pub async fn load() -> impl IntoResponse {
    let topologies = compiler::compile_root(&kit::pwd(), true);
    cache::write("root", &serde_json::to_string(&topologies).unwrap()).await;
    let mut functors  = Vec::from_iter(topologies.keys().cloned());
    functors.sort();
    let t = ListTemplate {
        namespace: String::from(""),
        items: functors
    };
    Html(t.render().unwrap())
}

pub async fn list() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let mut functors  = Vec::from_iter(topologies.keys().cloned());
    functors.sort();
    let t = ListTemplate {
        namespace: String::from(""),
        items: functors
    };
    Html(t.render().unwrap())
}
