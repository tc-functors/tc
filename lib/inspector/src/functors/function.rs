use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};

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


pub async fn vars(Path((root, namespace, name)): Path<(String, String, String)>) -> impl IntoResponse {

    let temp = DataTemplate {
        definition: name
    };
    Html(temp.render().unwrap())
}

pub async fn permissions(Path((root, namespace, name)): Path<(String, String, String)>) -> impl IntoResponse {

    let temp = DataTemplate {
        definition: name
    };
    Html(temp.render().unwrap())
}

pub async fn definition(Path((root, namespace, name)): Path<(String, String, String)>) -> impl IntoResponse {

    let temp = DataTemplate {
        definition: name
    };
    Html(temp.render().unwrap())
}
