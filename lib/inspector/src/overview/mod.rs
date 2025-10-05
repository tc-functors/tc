mod channels;
mod diagram;
mod events;
mod functions;
mod functors;
mod mutations;
mod nodes;
mod queues;
mod routes;
mod states;
mod pages;


use crate::Store;
use axum::{
    Router,
    routing::get,
};

// fragments
pub fn list_routes(store: &Store) -> Router {
    Router::new()
        .route("/hx/overview/functors", get(functors::list))
        .route("/hx/overview/functions", get(functions::list))
        .route("/hx/overview/nodes", get(nodes::list))
        .route("/hx/overview/events", get(events::list))
        .route("/hx/overview/channels", get(channels::list))
        .route("/hx/overview/queues", get(queues::list))
        .route("/hx/overview/routes", get(routes::list))
        .route("/hx/overview/states", get(states::list))
        .route("/hx/overview/mutations", get(mutations::list))
        .route("/hx/overview/pages", get(pages::list))
        .route("/hx/overview/diagram", get(diagram::render))
        .with_state(store.clone())
}
