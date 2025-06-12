use askama::Template;
use axum::{
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
};

#[derive(Template)]
#[template(path = "functor/channels.html")]
struct ListTemplate {
    root: String,
    namespace: String,
}

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = ListTemplate {
        root: root,
        namespace: namespace,
    };
    Html(temp.render().unwrap())
}
