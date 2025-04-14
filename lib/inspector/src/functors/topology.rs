use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use crate::cache;

#[derive(Template)]
#[template(path = "functors/topology.html")]
struct ViewTemplate {
    topology: String,
}


pub async fn view(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&root, &namespace).await;
    if let Some(t) = f {
        let temp = ViewTemplate {
            topology: t.to_str()
        };
        Html(temp.render().unwrap())

    } else {
        let temp = ViewTemplate {
            topology: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}
