use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "deployments/view/functors.html")]
struct FunctorsTemplate {
    items: Vec<String>
}

pub async fn view() -> impl IntoResponse {
    let t = FunctorsTemplate {
        items: vec![]
    };
    Html(t.render().unwrap())
}


#[derive(Template)]
#[template(path = "deployments/tabs/functors.html")]
struct TabTemplate {
    envs: Vec<String>,
}

pub async fn tab() -> impl IntoResponse {
   let t = TabTemplate {
       envs: vec![]
    };
    Html(t.render().unwrap())
}
