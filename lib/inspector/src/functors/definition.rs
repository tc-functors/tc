use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use crate::cache;

#[derive(Template)]
#[template(path = "functors/definition.html")]
struct ViewTemplate {
    name: String,
    definition: String,
    topology: String,
    flow: String
}

fn lookup_definition(dir: &str) -> String {
    let f = format!("{}/topology.yml", dir);
    kit::slurp(&f)
}

pub async fn view(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&namespace, &namespace).await;
    if let Some(t) = f {
        let definition = lookup_definition(&t.dir);
        let temp = ViewTemplate {
            name: namespace,
            definition: definition,
            topology: t.to_str(),
            flow: String::from("")
        };
        Html(temp.render().unwrap())

    } else {
        let temp = ViewTemplate {
            name: namespace,
            definition: String::from("test"),
            topology: String::from("test"),
            flow: String::from("")
        };
        Html(temp.render().unwrap())
    }
}
