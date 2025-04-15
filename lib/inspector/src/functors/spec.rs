use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use compiler::spec::FunctionSpec;

#[derive(Template)]
#[template(path = "functors/spec.html")]
struct ViewTemplate {
    topology: String,
    function: String
}


pub async fn view() -> impl IntoResponse {
    let fspec = doku::to_json::<FunctionSpec>();
    let temp = ViewTemplate {
        topology: String::from(""),
        function: fspec
    };
    Html(temp.render().unwrap())
}
