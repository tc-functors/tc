use crate::cache;
use askama::Template;
use axum::{
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
};
use composer::Route;
use std::collections::HashMap;

struct Item {
    path: String,
    method: String,
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
    let mut xs: Vec<Item> = vec![];
    for (_, route) in routes {
        let item = Item {
            path: route.path,
            method: route.method,
            kind: route.entity.to_str(),
            target: route.target_name,
        };
        xs.push(item);
    }
    xs
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
