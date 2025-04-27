use askama::Template;
use axum::{
    Router,
    response::{
        Html,
        IntoResponse,
    },
    routing::get,
};
use compiler::spec::{
    FunctionSpec,
    RuntimeInfraSpec,
};
use configurator::Config;

#[derive(Template)]
#[template(path = "specs/index.html")]
struct PageTemplate {
    context: String,
    entity: String,
    data: String,
}

pub async fn function_page() -> impl IntoResponse {
    let fspec = doku::to_json::<FunctionSpec>();
    let temp = PageTemplate {
        context: String::from("specs"),
        entity: String::from("function"),
        data: fspec,
    };
    Html(temp.render().unwrap())
}

pub async fn topology_page() -> impl IntoResponse {
    let fspec = doku::to_json::<FunctionSpec>();
    let temp = PageTemplate {
        context: String::from("specs"),
        entity: String::from("topology"),
        data: fspec,
    };
    Html(temp.render().unwrap())
}

pub async fn infra_page() -> impl IntoResponse {
    let fspec = doku::to_json::<RuntimeInfraSpec>();
    let temp = PageTemplate {
        context: String::from("specs"),
        entity: String::from("infrastructure"),
        data: fspec,
    };
    Html(temp.render().unwrap())
}

pub async fn config_page() -> impl IntoResponse {
    let cspec = doku::to_toml::<Config>();
    //let config = Config::new(None, "dev");
    //let cfg = serde_json::to_string(&config).unwrap();
    let temp = PageTemplate {
        context: String::from("specs"),
        entity: String::from("config"),
        data: cspec,
    };
    Html(temp.render().unwrap())
}

pub fn routes() -> Router {
    Router::new()
        .route("/specs", get(function_page))
        .route("/specs/function", get(function_page))
        .route("/specs/topology", get(topology_page))
        .route("/specs/infra", get(infra_page))
        .route("/specs/config", get(config_page))
}
