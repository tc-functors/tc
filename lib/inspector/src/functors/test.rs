use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use crate::cache;

#[derive(Template)]
#[template(path = "functors/test_form.html")]
struct FormTemplate {
}


pub async fn form(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = FormTemplate {
    };
    Html(temp.render().unwrap())
}
