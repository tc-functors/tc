use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

use crate::cache;

#[derive(Template)]
#[template(path = "functors/definition.html")]
struct ViewTemplate {
    item: String
}

pub async fn view(Path((root, namespace, _id)): Path<(String, String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&root, &namespace).await;
    if let Some(t) = f {
        let temp = ViewTemplate {
            item: t.to_str()
        };
        Html(temp.render().unwrap())

    } else {
        let temp = ViewTemplate {
            item: String::from("test")
        };
        Html(temp.render().unwrap())
    }
}
