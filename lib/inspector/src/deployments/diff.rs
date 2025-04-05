use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "deployments/view/diff.html")]
struct DiffTemplate {
    left: String,
    right: String
}

pub async fn view() -> impl IntoResponse {
    let t = DiffTemplate {
        left: String::from("-layers"),
        right: String::from("+layers")
    };
    Html(t.render().unwrap())
}

#[derive(Template)]
#[template(path = "deployments/tabs/diff.html")]
struct TabTemplate {
    envs: Vec<String>,
}

pub async fn tab() -> impl IntoResponse {
   let t = TabTemplate {
       envs: vec![]
    };
    Html(t.render().unwrap())
}
