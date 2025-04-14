use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use crate::cache;

#[derive(Template)]
#[template(path = "functors/index.html")]
struct PageTemplate {
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

pub async fn page(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&namespace, &namespace).await;
    if let Some(t) = f {
        let definition = lookup_definition(&t.dir);
        let temp = PageTemplate {
            context: String::from("functors"),
            root: root,
            namespace: namespace,
            definition: definition,
            topology: t.to_str()
        };
        Html(temp.render().unwrap())

    } else {
        let temp = PageTemplate {
            context: String::from("functors"),
            root: root,
            namespace: namespace,
            definition: String::from("test"),
            topology: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}

// view


#[derive(Template)]
#[template(path = "functors/definition.html")]
struct ViewTemplate {
    definition: String,
}

pub async fn view(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&namespace, &namespace).await;
    if let Some(t) = f {
        let definition = lookup_definition(&t.dir);
        let temp = ViewTemplate {
            definition: definition,
        };
        Html(temp.render().unwrap())

    } else {
        let temp = ViewTemplate {
            definition: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}
