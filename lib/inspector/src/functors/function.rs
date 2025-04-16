use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    Form,
};

use serde::Deserialize;
use crate::cache;

#[derive(Template)]
#[template(path = "functors/function/build.html")]
struct FlowTemplate {
}

pub async fn build(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {

    let temp = FlowTemplate {
    };
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functors/code.html")]
struct DataTemplate {
    definition: String
}


#[derive(Deserialize, Debug)]
pub struct FunctionInput {
    pub function: String,
}

pub async fn compile(
    Path((root, namespace)): Path<(String, String)>,
    Form(payload): Form<FunctionInput>
) -> impl IntoResponse {
    let FunctionInput { function } = payload;
    let function = cache::find_function(&root, &namespace, &function).await;

    let definition = if let Some(f) = function {
        serde_json::to_string_pretty(&f).unwrap()
    } else {
        String::from("")
    };
    let temp = DataTemplate {
        definition: definition
    };
    Html(temp.render().unwrap())
}
