use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    extract::Path,
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
mod topology;
mod functions;
mod mutations;
mod routes;
mod queues;
mod events;
mod states;
mod channels;

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
#[template(path = "functor/index.html")]
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

pub async fn main_page(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = IndexTemplate {
        root: root,
        namespace: namespace,
    };
    Html(temp.render().unwrap())
}


pub fn page_routes() -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/functors", get(index_page))
        .route("/functor/{:root}/{:namespace}", get(main_page))
}

pub fn list_routes() -> Router {
    Router::new()
        .route(
            "/hx/functor/topology/{:root}/{:namespace}",
            get(topology::definition),
        )
        .route(
            "/hx/functor/functions/{:root}/{:namespace}",
            get(functions::list),
        )
        .route(
            "/hx/functor/mutations/{:root}/{:namespace}",
            get(mutations::list),
        )
        .route(
            "/hx/functor/states/{:root}/{:namespace}",
            get(states::list),
        )
        .route(
            "/hx/functor/events/{:root}/{:namespace}",
            get(events::list),
        )
        .route(
            "/hx/functor/routes/{:root}/{:namespace}",
            get(routes::list),
        )
        .route(
            "/hx/functor/channels/{:root}/{:namespace}",
            get(channels::list),
        )
        .route(
            "/hx/functor/queues/{:root}/{:namespace}",
            get(queues::list),
        )
}


pub fn topology_routes() -> Router {
    Router::new()
        .route(
            "/hx/functor/topology/compile/{:root}/{:namespace}",
            post(topology::compile),
        )
        .route("/hx/functor/topology/flow/{:root}/{:namespace}",
               post(topology::flow))
        .route(
            "/hx/functor/topology/sandbox-form/{:root}/{:namespace}",
            post(topology::sandbox),
        )
        .route(
            "/hx/functor/topology/test-form/{:root}/{:namespace}",
            post(topology::test),
        )
}

pub fn function_routes() -> Router {
    Router::new()
        .route(
            "/hx/functor/function/build/{:root}/{:namespace}",
            post(functions::build),
        )
        .route(
            "/hx/functor/function/compile/{:root}/{:namespace}",
            post(functions::compile),
        )
        .route(
            "/hx/functor/function/permissions/{:root}/{:namespace}",
            post(functions::permissions),
        )
}

pub fn mutation_routes() -> Router {
    Router::new().route(
        "/hx/functor/mutation/compile/{:root}/{:namespace}",
        post(mutations::compile),
    )
}
