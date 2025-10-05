use crate::Store;
use askama::Template;
use axum::{
    extract::{
        Path,
        State,
    },
    response::{
        Html,
        IntoResponse,
    },
};

#[derive(Template)]
#[template(path = "functor/queues.html")]
struct ListTemplate {
    root: String,
    namespace: String,
}

pub async fn list(
    State(_store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let temp = ListTemplate {
        root: root,
        namespace: namespace,
    };
    Html(temp.render().unwrap())
}
