use crate::cache;
use askama::Template;
use axum::{
    Router,
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
    routing::get,
};

#[derive(Template)]
#[template(path = "list.html")]
struct ListTemplate {
    root: String,
    namespace: String,
    items: Vec<String>,
}

pub async fn _load() -> impl IntoResponse {
    let topologies = compiler::compile_root(&kit::pwd(), true);
    cache::write("root", &serde_json::to_string(&topologies).unwrap()).await;
    let mut functors = Vec::from_iter(topologies.keys().cloned());
    functors.sort();
    let t = ListTemplate {
        root: compiler::topology_name(&kit::pwd()),
        namespace: String::from(""),
        items: functors,
    };
    Html(t.render().unwrap())
}

pub async fn functors(Path((_root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let topologies = cache::find_all_topologies().await;
    let mut functors = Vec::from_iter(topologies.keys().cloned());
    functors.sort();
    let t = ListTemplate {
        root: compiler::topology_name(&kit::pwd()),
        namespace: namespace,
        items: functors,
    };
    Html(t.render().unwrap())
}

pub fn routes() -> Router {
    Router::new().route("/hx/functors/list/{:root}/{:namespace}", get(functors))
}
