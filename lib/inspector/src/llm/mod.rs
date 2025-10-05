mod model;
use crate::Store;
use askama::Template;
use axum::{
    Form,
    extract::State,
    Router,
    routing::{post, get},
    response::{
        Html,
        IntoResponse,
    },
};
use json_escape::escape_str;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "llm/response.html")]
struct ResponseTemplate {
    data: String
}

#[derive(Deserialize, Debug)]
pub struct FormInput {
    pub text: String,
}

pub async fn prompt(State(_store): State<Store>, Form(f): Form<FormInput>) -> impl IntoResponse {
    let input = &f.text;
    let response = if input == "test" {
        String::from("# Testing. No data")
    } else {
        model::send(&f.text).await
    };

    let r = escape_str(&response);
    let temp = ResponseTemplate { data: r.to_string() };
    Html(temp.render().unwrap())
}


pub async fn clear(State(_store): State<Store>) -> impl IntoResponse {
    let response = String::from("");
    let r = escape_str(&response);
    let temp = ResponseTemplate { data: r.to_string() };
    Html(temp.render().unwrap())
}

pub fn render_routes(store: &Store) -> Router {
    Router::new()
        .route("/hx/llm/prompt", post(prompt))
        .route("/hx/llm/clear", get(clear))
        .with_state(store.clone())
}
