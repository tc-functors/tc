use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

struct Event {
   name: String
}

#[derive(Template)]
#[template(path = "definitions/fragments/events.html")]
struct EventsTemplate {
    items: Vec<Event>
 }

pub async fn list(Path(_id): Path<String>) -> impl IntoResponse {
    let t = EventsTemplate { items: vec![] };
    Html(t.render().unwrap())
}
