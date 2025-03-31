use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use std::collections::HashMap;
use compiler::{Topology, Event};
use crate::cache;

struct Item {
    namespace: String,
    name: String,
    rule_name: String,
    pattern: String,
    targets: HashMap<String, String>
}


fn build_events(namespace: &str, evs: HashMap<String, Event>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (_, event) in evs {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &event.targets {
            targets.insert(t.kind.to_str(), t.name.clone());
        }
        let e = Item {
            namespace: namespace.to_string(),
            name: event.name.clone(),
            rule_name: event.rule_name.clone(),
            pattern: serde_json::to_string(&event.pattern).unwrap(),
            targets: targets
        };
        xs.push(e);
    }
    xs
}

fn build(topologies: HashMap<String, Topology>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];

    for (_, topology) in topologies {
        let fns = build_events(&topology.namespace, topology.events);
        xs.extend(fns);
        for (_, node) in topology.nodes {
            let fns = build_events(&node.namespace, node.events);
            xs.extend(fns)
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "definitions/list/events.html")]
struct EventsTemplate {
    items: Vec<Item>
}

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let events = cache::find_events(&root, &namespace).await;
    let temp = EventsTemplate {
        items: build_events(&namespace, events)
    };
    Html(temp.render().unwrap())
}

pub async fn list_all() -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let events = build(topologies);
    let temp = EventsTemplate {
        items: events
    };
    Html(temp.render().unwrap())
}

// visualization

async fn build_participants(roots: &Vec<String>) -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    for root in roots {
        let x = format!("participant {}", kit::split_first(&root, "-"));
        xs.push(x);
    }
    xs
}

async fn build_mermaid_str() -> Vec<String> {
    let events = cache::find_all_events().await;
    let mut xs: Vec<String> = vec![];
    let roots = cache::find_root_namespaces().await;
    //let parts = build_participants(&roots).await;

    //xs.extend(parts);

    for (_, event) in events {
        for t in event.targets {
            let producer = t.producer_ns;
            let consumer = t.consumer_ns;

            let target_name = &t.name
                .replace("{{namespace}}_", "")
                .replace("{{namespace}}-", "")
                .replace("_{{sandbox}}", "")
                .replace("-{{sandbox}}", "");

            if roots.contains(&consumer) && roots.contains(&producer) {
                let c = kit::split_first(&consumer, "-");
                let x = format!("{}->>{}: {}", producer, &c, &event.name);
                xs.push(x);
                let note = format!("note left of {}: λ {}", &c, target_name);
                xs.push(note);
            }
        }
    }
    xs
}


#[derive(Template)]
#[template(path = "definitions/visual/events.html")]
struct VisualTemplate {
    items: Vec<String>
}

pub async fn visualize() -> impl IntoResponse {
    let xs = build_mermaid_str().await;

    let temp = VisualTemplate {
        items: xs
    };
    Html(temp.render().unwrap())
}
