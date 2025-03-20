use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use std::collections::HashMap;
use compiler::Topology;


struct Event {
    namespace: String,
    name: String,
    rule_name: String,
    pattern: String,
    targets: HashMap<String, String>
}


fn build(topology: &Topology) -> Vec<Event> {
    let mut xs: Vec<Event> = vec![];
    for (_, event) in &topology.events {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &event.targets {
            targets.insert(t.kind.to_str(), t.name.clone());
        }
        let e = Event {
            namespace: topology.namespace.clone(),
            name: event.name.clone(),
            rule_name: event.rule_name.clone(),
            pattern: serde_json::to_string(&event.pattern).unwrap(),
            targets: targets
        };
        xs.push(e);
    }
    xs
}

fn build_all(topologies: HashMap<String, Topology>) -> Vec<Event> {
    let mut xs: Vec<Event> = vec![];

    for (_, topology) in topologies {
        let fns = build(&topology);
        xs.extend(fns);
        for (_, node) in topology.nodes {
            let fns = build(&node);
            xs.extend(fns)
        }
    }
    xs
}

#[derive(Template)]
#[template(path = "definitions/list/events.html")]
struct EventsTemplate {
    items: Vec<Event>
 }

pub async fn list(Path(id): Path<String>) -> impl IntoResponse {
    let topologies = cache::read_topologies("root").await;

    if &id == "all" {

        let xs = build_all(topologies);
        let temp = EventsTemplate {
            items: xs
        };
        Html(temp.render().unwrap())

    } else {
        let maybe_topology = topologies.get(&id);

        if let Some(t) = maybe_topology {
            tracing::debug!("Found topology");
            let temp = EventsTemplate {
                items: build(&t)
            };
            Html(temp.render().unwrap())
        } else {
            let temp = EventsTemplate {
                items: vec![]
            };
            Html(temp.render().unwrap())
        }
    }
}
