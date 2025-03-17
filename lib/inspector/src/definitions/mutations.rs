use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

struct Route {
    name: String
}

#[derive(Template)]
#[template(path = "definitions/fragments/mutations.html")]
struct RoutesTemplate {
    items: Vec<Route>
 }

pub async fn list(Path(_id): Path<String>) -> impl IntoResponse {
    let t = RoutesTemplate { items: vec![] };
    Html(t.render().unwrap())
}
