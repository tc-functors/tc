use askama::Template;
use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
mod channels;
mod events;
mod functions;
mod mutations;
mod queues;
mod routes;
mod states;
mod topology;

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
    functions: usize,
    events: usize,
    routes: usize,
    queues: usize,
    channels: usize,
    mutations: usize,
    states: usize,
}

pub async fn index_page() -> impl IntoResponse {
    let name = topology::name_of();
    HtmlTemplate(IndexTemplate {
        root: String::from(&name),
        namespace: name,
        functions: 0,
        events: 0,
        routes: 0,
        queues: 0,
        channels: 0,
        mutations: 0,
        states: 0,
    })
}

pub async fn main_page(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let count = topology::count_of(&root, &namespace).await;
    let temp = IndexTemplate {
        root: root,
        namespace: namespace,
        functions: count.functions,
        events: count.events,
        routes: count.routes,
        queues: count.queues,
        channels: count.channels,
        mutations: count.mutations,
        states: count.states,
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
        .route("/hx/functor/states/{:root}/{:namespace}", get(states::list))
        .route("/hx/functor/events/{:root}/{:namespace}", get(events::list))
        .route("/hx/functor/routes/{:root}/{:namespace}", get(routes::list))
        .route(
            "/hx/functor/channels/{:root}/{:namespace}",
            get(channels::list),
        )
        .route("/hx/functor/queues/{:root}/{:namespace}", get(queues::list))
}

pub fn topology_routes() -> Router {
    Router::new()
        .route(
            "/hx/functor/topology/compile/{:root}/{:namespace}",
            post(topology::compile),
        )
        .route(
            "/hx/functor/topology/flow/{:root}/{:namespace}",
            post(topology::flow),
        )
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
