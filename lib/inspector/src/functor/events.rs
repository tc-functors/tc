use crate::Store;
use askama::Template;
use axum::{
    extract::{
        Path,
        State,
    },
    response::{
        Html,
        IntoResponse,
    },
};
use composer::Event;
use std::collections::HashMap;

struct Item {
    name: String,
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
    let mut xs: Vec<Item> = vec![];
    for (name, ev) in events {
        for target in ev.targets {
            let item = Item {
                name: name.clone(),
                kind: target.entity.to_str(),
                target: target.name,
            };
            xs.push(item);
        }
    }
    xs
}

pub async fn list(
    State(store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let events = store.find_events(&root, &namespace).await;
    let items = build(events);
    let temp = ListTemplate {
        root: root,
        namespace: namespace,
        items: items,
    };
    Html(temp.render().unwrap())
}
