use askama::Template;
use axum::{
    extract::Path,
    routing::{get, post},
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod loader;
mod flow;
mod definition;
mod topology;
mod sandbox;
mod test;

pub struct HtmlTemplate<T>(pub T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "functors/index.html")]
struct IndexTemplate {
    root: String,
    namespace: String,
    context: String,
    definition: String,
    flow: String,
    topology: String
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        root: String::from("default"),
        namespace: String::from("default"),
        context: String::from("functors"),
        definition: String::from(""),
        flow: String::from(""),
        topology: String::from("")
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/functors", get(index_page))
        .route("/functor/{:root}/{:namespace}",
               get(definition::page))
        .route("/hx/functors/load", post(loader::load))
        .route("/hx/functors/list/{:root}/{:namespace}",
               get(loader::list))
        .route("/hx/functor/definition/{:root}/{:namespace}",
               post(definition::view))
        .route("/hx/functor/topology/{:root}/{:namespace}",
               post(topology::view))
        .route("/hx/functor/flow/{:root}/{:namespace}",
               post(flow::view))
        .route("/hx/functor/sandbox-form/{:root}/{:namespace}",
               post(sandbox::form))
        .route("/hx/functor/test-form/{:root}/{:namespace}",
               post(test::form))
}
