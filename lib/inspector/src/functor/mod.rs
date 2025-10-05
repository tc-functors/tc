use crate::{
    Store,
    counter,
};
use askama::Template;
use axum::{
    Router,
    extract::{
        Path,
        State,
    },
    response::{
        Html,
        IntoResponse,
    },
    routing::{
        get,
        post,
    },
};

mod channels;
mod events;
mod functions;
mod mutations;
mod queues;
mod routes;
mod states;
mod topology;

#[derive(Template)]
#[template(path = "functor/main.html")]
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

pub async fn main_page(
    State(store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let count = counter::count_of(&store, &root, &namespace).await;
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

pub fn main_routes(store: &Store) -> Router {
    Router::new()
        .route("/functor/{:root}/{:namespace}", get(main_page))
        .with_state(store.clone())
}

pub fn list_routes(store: &Store) -> Router {
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
        .with_state(store.clone())
}

pub fn topology_routes(store: &Store) -> Router {
    Router::new()
        .route(
            "/hx/functor/topology/compose/{:root}/{:namespace}",
            post(topology::compose),
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
        .with_state(store.clone())
}

pub fn function_routes(store: &Store) -> Router {
    Router::new()
        .route(
            "/hx/functor/function/build/{:root}/{:namespace}",
            post(functions::build),
        )
        .route(
            "/hx/functor/function/compose/{:root}/{:namespace}",
            post(functions::compose),
        )
        .route(
            "/hx/functor/function/permissions/{:root}/{:namespace}",
            post(functions::permissions),
        )
        .with_state(store.clone())
}

pub fn mutation_routes(store: &Store) -> Router {
    Router::new()
        .route(
            "/hx/functor/mutation/compile/{:root}/{:namespace}",
            post(mutations::compile),
        )
        .with_state(store.clone())
}

async fn list_namespaces(store: &Store) -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    let topologies = store.list_topologies().await;
    for t in topologies {
        xs.push(t.namespace.clone());
    }
    xs
}

#[derive(Template)]
#[template(path = "functor/sidebar.html")]
struct ListTemplate {
    namespace: String,
    items: Vec<String>,
}

pub async fn list_functors(
    State(store): State<Store>,
    Path((_root, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let t = ListTemplate {
        namespace: namespace,
        items: list_namespaces(&store).await
    };
    Html(t.render().unwrap())
}

//
pub fn sidebar_routes(store: &Store) -> Router {
    Router::new()
        .route("/hx/functors/list/{:root}/{:namespace}", get(list_functors))
        .with_state(store.clone())
}
