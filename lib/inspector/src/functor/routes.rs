use crate::cache;
use askama::Template;
use axum::{
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
};
use compiler::{
    Route,
};
use std::collections::HashMap;

struct Item {
    path: String,
    kind: String,
    target: String,
}

#[derive(Template)]
#[template(path = "functor/routes.html")]
struct ListTemplate {
    root: String,
    namespace: String,
    items: Vec<Item>,
}


fn build(routes: HashMap<String, Route>) -> Vec<Item> {
    vec![]
}

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let routes = cache::find_routes(&root, &namespace).await;
    let items = build(routes);
    let temp = ListTemplate {
        root: root,
        namespace: namespace,
        items: items,
    };
    Html(temp.render().unwrap())
}
