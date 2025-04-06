mod event;
mod route;
mod function;
mod mutation;
mod node;
mod permission;
mod root;
mod page;

use axum::{
    routing::{get, post},
    Router,
};

pub fn page_routes() -> Router {
    Router::new()
        .route("/",
               get(page::definitions))
        .route("/definitions",
               get(page::definitions))

        .route("/definition/view/{:root}",
               get(page::view_root))
        .route("/definition/view/{:root}/{:namespace}/{:entity}/{:id}",
               get(page::view_entity))

        .route("/definitions/list",
               get(page::definitions))

        .route("/definitions/list/{:entity}/all",
               get(page::list_all))
        .route("/definitions/list/{:root}/{:entity}",
               get(page::list_root))
        .route("/definitions/list/{:root}/{:namespace}/{:entity}",
               get(page::list_ns))
}

// fragments

pub fn list_routes() -> Router {
    Router::new()
        .route("/hx/def/list",
               get(root::list_all))
        .route("/hx/def/list/all/all/functions",
               get(function::list_all))
        .route("/hx/def/list/all/all/nodes",
               get(node::list_all))
        .route("/hx/def/list/all/all/events",
               get(event::list_all))
        .route("/hx/def/list/all/all/routes",
               get(route::list_all))
        .route("/hx/def/list/all/all/mutations",
               get(mutation::list_all))
        .route("/hx/def/list/all/all/permissions",
               get(permission::list_all))

        .route("/hx/def/list/{:root}/{:namespace}/functions",
               get(function::list))
        .route("/hx/def/list/{:root}/{:namespace}/nodes",
               get(node::list))
        .route("/hx/def/list/{:root}/{:namespace}/events",
               get(event::list))
        .route("/hx/def/list/{:root}/{:namespace}/mutations",
               get(mutation::list))
        .route("/hx/def/list/{:root}/{:namespace}/routes",
               get(route::list))
}


pub fn view_routes() -> Router {
    Router::new()
        .route("/hx/def/{:root}/{:namespace}/function/{:id}/view",
               get(function::view))
        .route("/hx/def/{:root}/{:namespace}/node/{:id}/view",
               get(node::view))
}


pub fn post_routes() -> Router {
    Router::new()
        .route("/hx/def/compile",
               post(root::compile))

}
