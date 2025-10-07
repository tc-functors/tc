use crate::Store;
use askama::Template;
use axum::{
    extract::State,
    response::{
        Html,
        IntoResponse,
    },
};

use composer::{
    Topology,
};

fn build(topologies: Vec<Topology>) -> String {

    let mut s: String = String::from("");

    for topology in topologies {
        let name = topology.namespace;
        let sg = format!(r#"
    subgraph {name}
"#);
        s.push_str(&sg);

        for (_, node) in topology.nodes {
            let name = node.namespace;
            let sg = format!(r#"
{name}
"#);
            s.push_str(&sg);
        }
        let sg = format!(r#"
end
"#);
            s.push_str(&sg);
    }

    format!(r#"
    flowchart TB {s}
"#)
}

#[derive(Template)]
#[template(path = "overview/diagram.html")]
struct DiagramTemplate {
    data: String
}

pub async fn render(State(store): State<Store>) -> impl IntoResponse {
    let topologies = store.list_topologies().await;
    let chart = build(topologies);
    println!("{}", &chart);
    let temp = DiagramTemplate { data: chart };
    Html(temp.render().unwrap())
}
