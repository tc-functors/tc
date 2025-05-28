use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{
        Html,
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
mod function;
mod mutation;
mod topology;
mod entity;

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
#[template(path = "index.html")]
struct IndexTemplate {
    root: String,
    namespace: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        root: String::from("default"),
        namespace: String::from("default"),
    })
}

pub fn page_routes() -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/functors", get(index_page))
        .route("/functor/{:root}/{:namespace}", get(entity::page))
}

pub fn entity_routes() -> Router {
    Router::new()
        .route(
            "/hx/functor/definition/{:root}/{:namespace}",
            get(entity::definition),
        )
        .route(
            "/hx/functor/functions/{:root}/{:namespace}",
            get(entity::functions),
        )
        .route(
            "/hx/functor/mutations/{:root}/{:namespace}",
            get(entity::mutations),
        )
        .route(
            "/hx/functor/states/{:root}/{:namespace}",
            get(entity::states),
        )
}


pub fn topology_routes() -> Router {
    Router::new()
        .route(
            "/hx/functor/compile/{:root}/{:namespace}",
            post(topology::compile),
        )
        .route("/hx/functor/flow/{:root}/{:namespace}",
               post(topology::flow))
        .route(
            "/hx/functor/sandbox-form/{:root}/{:namespace}",
            post(topology::sandbox),
        )
        .route(
            "/hx/functor/test-form/{:root}/{:namespace}",
            post(topology::test),
        )
}

pub fn function_routes() -> Router {
    Router::new()
        .route(
            "/hx/function/build/{:root}/{:namespace}",
            post(function::build),
        )
        .route(
            "/hx/function/compile/{:root}/{:namespace}",
            post(function::compile),
        )
        .route(
            "/hx/function/permissions/{:root}/{:namespace}",
            post(function::permissions),
        )
}

pub fn mutation_routes() -> Router {
    Router::new().route(
        "/hx/mutation/compile/{:root}/{:namespace}",
        post(mutation::compile),
    )
}
