use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

struct Mutation {
    name: String
}

#[derive(Template)]
#[template(path = "definitions/fragments/mutations.html")]
struct MutationsTemplate {
    items: Vec<Mutation>
 }

pub async fn list(Path(_id): Path<String>) -> impl IntoResponse {
    let t = MutationsTemplate { items: vec![] };
    Html(t.render().unwrap())
}
