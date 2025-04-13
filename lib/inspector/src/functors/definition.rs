use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use crate::cache;

#[derive(Template)]
#[template(path = "functors/index.html")]
struct ViewTemplate {
    context: String,
    root: String,
    namespace: String,
    definition: String,
    topology: String,
}

fn lookup_definition(dir: &str) -> String {
    let f = format!("{}/topology.yml", dir);
    println!("Loading {}", &f);
    kit::slurp(&f)
}

pub async fn view(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&namespace, &namespace).await;
    if let Some(t) = f {
        let definition = lookup_definition(&t.dir);
        let temp = ViewTemplate {
            context: String::from("functors"),
            root: root,
            namespace: namespace,
            definition: definition,
            topology: t.to_str()
        };
        Html(temp.render().unwrap())

    } else {
        let temp = ViewTemplate {
            context: String::from("functors"),
            root: root,
            namespace: namespace,
            definition: String::from("test"),
            topology: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}
