pub mod functors;
pub mod events;
pub mod routes;
pub mod functions;
pub mod mutations;
pub mod nodes;

use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    Form,
};

use serde_derive::Deserialize;

#[derive(Template)]
#[template(path = "definitions/fragments/compile_form.html")]
struct CompileTemplate {}

#[derive(Template)]
#[template(path = "definitions/fragments/test_form.html")]
struct TestTemplate {}

#[derive(Template)]
#[template(path = "definitions/fragments/visualize_form.html")]
struct VisualizeTemplate {}

#[derive(Template)]
#[template(path = "definitions/fragments/sandbox_form.html")]
struct SandboxTemplate {}


#[derive(Deserialize, Debug)]
pub struct ActionInput {
    pub action: String,
}


pub async fn show_action_form(Form(payload): Form<ActionInput>) -> impl IntoResponse {

    let ActionInput { action } = payload;

    match action.as_ref() {
        "test" => {
            let t = TestTemplate {};
            Html(t.render().unwrap())
        },
        "compile" => {
            let t = CompileTemplate {};
            Html(t.render().unwrap())
        },
        "visualize" => {
            let t = VisualizeTemplate {};
            Html(t.render().unwrap())
        },
        "create-sandbox" => {
            let t = SandboxTemplate {};
            Html(t.render().unwrap())
        },
        _  => {
            let t = TestTemplate {};
            Html(t.render().unwrap())
        }
    }
}
