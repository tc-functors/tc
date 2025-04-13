mod event;
mod route;
mod function;
mod mutation;
mod node;
mod permission;
mod root;
mod page;
mod graph;

use axum::{
    routing::{get, post},
    Router,
};

pub fn page_routes() -> Router {
    Router::new()
        .route("/overview",
               get(page::index))

        .route("/overview/view/{:root}",
               get(page::view_root))
        .route("/overview/view/{:root}/{:namespace}/{:entity}/{:id}",
               get(page::view_entity))

        .route("/overview/list",
               get(page::index))

        .route("/overview/list/{:entity}/all",
               get(page::list_all))
        .route("/overview/list/{:root}/{:entity}",
               get(page::list_root))
        .route("/overview/list/{:root}/{:namespace}/{:entity}",
               get(page::list_ns))
}

// fragments

pub fn list_routes() -> Router {
    Router::new()
        .route("/hx/overview/list",
               get(root::list_all))
        .route("/hx/overview/list/all/all/functors",
               get(root::list_all))
        .route("/hx/overview/list/all/all/functions",
               get(function::list_all))
        .route("/hx/overview/list/all/all/nodes",
               get(node::list_all))
        .route("/hx/overview/list/all/all/events",
               get(event::list_all))
        .route("/hx/overview/list/all/all/routes",
               get(route::list_all))
        .route("/hx/overview/list/all/all/mutations",
               get(mutation::list_all))
        .route("/hx/overview/list/all/all/permissions",
               get(permission::list_all))

        .route("/hx/overview/list/{:root}/{:namespace}/functions",
               get(function::list))
        .route("/hx/overview/list/{:root}/{:namespace}/nodes",
               get(node::list))
        .route("/hx/overview/list/{:root}/{:namespace}/events",
               get(event::list))
        .route("/hx/overview/list/{:root}/{:namespace}/mutations",
               get(mutation::list))
        .route("/hx/overview/list/{:root}/{:namespace}/routes",
               get(route::list))
}


pub fn view_routes() -> Router {
    Router::new()
        .route("/hx/overview/view/{:root}/{:namespace}/function/{:id}",
               get(function::view))
}


pub fn post_routes() -> Router {
    Router::new()
        .route("/hx/overview/flow",
               post(graph::flow))
        .route("/hx/overview/sequence",
               post(graph::sequence))

}
