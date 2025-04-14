use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "functors/test_form.html")]
struct FormTemplate {
}


pub async fn form(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = FormTemplate {
    };
    Html(temp.render().unwrap())
}
