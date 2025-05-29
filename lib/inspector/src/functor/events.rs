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
    Event,
    TopologySpec,
};
use std::collections::HashMap;

struct Item {
    name: String,
    pattern: String,
    kind: String,
    target: String,
}

#[derive(Template)]
#[template(path = "functor/events.html")]
struct ListTemplate {
    root: String,
    namespace: String,
    items: Vec<Item>,
}

fn build(events: HashMap<String, Event>) -> Vec<Item> {
    vec![]
}

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let events = cache::find_events(&root, &namespace).await;
    let items = build(events);
    let temp = ListTemplate {
        root: root,
        namespace: namespace,
        items: items,
    };
    Html(temp.render().unwrap())
}
