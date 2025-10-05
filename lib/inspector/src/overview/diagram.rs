use crate::Store;
use askama::Template;
use axum::{
    extract::State,
    response::{
        Html,
        IntoResponse,
    },
};

#[derive(Template)]
#[template(path = "overview/diagram.html")]
struct DiagramTemplate {}

pub async fn render(State(_store): State<Store>) -> impl IntoResponse {
    let temp = DiagramTemplate { };
    Html(temp.render().unwrap())
}
