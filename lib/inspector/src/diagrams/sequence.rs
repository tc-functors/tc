use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use crate::cache;

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
#[template(path = "diagrams/sequence.html")]
struct SequenceTemplate {
    items: Vec<String>
}

pub async fn generate() -> impl IntoResponse {
    let xs = build_mermaid_str().await;

    let temp = SequenceTemplate {
        items: xs
    };
    Html(temp.render().unwrap())
}
