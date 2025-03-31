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
        .route("/definitions/visualize/{:entity}",
               get(page::visualize))
        .route("/definition/{:root}",
               get(page::view_root_definition))
        .route("/definition/{:root}/{:namespace}/{:entity}/{:id}",
               get(page::view_entity_definition))
        .route("/definitions/{:entity}/all",
               get(page::list_all_definitions))
        .route("/definitions/{:root}/{:entity}",
               get(page::list_root_definitions))
        .route("/definitions/{:root}/{:namespace}/{:entity}",
               get(page::list_ns_definitions))
}

// fragments

pub fn list_routes() -> Router {
    Router::new()
        .route("/definitions/list",
               get(root::list_all))
        .route("/definitions/all/all/functions/list",
               get(function::list_all))
        .route("/definitions/all/all/nodes/list",
               get(node::list_all))
        .route("/definitions/all/all/events/list",
               get(event::list_all))
        .route("/definitions/all/all/routes/list",
               get(route::list_all))
        .route("/definitions/all/all/mutations/list",
               get(mutation::list_all))
        .route("/definitions/all/all/permissions/list",
               get(permission::list_all))

        .route("/definitions/{:root}/{:namespace}/functions/list",
               get(function::list))
        .route("/definitions/{:root}/{:namespace}/nodes/list",
               get(node::list))
        .route("/definitions/{:root}/{:namespace}/events/list",
               get(event::list))
        .route("/definitions/{:root}/{:namespace}/mutations/list",
               get(mutation::list))
        .route("/definitions/{:root}/{:namespace}/routes/list",
               get(route::list))
}

pub fn visualize_routes() -> Router {
    Router::new()
        .route("/definitions/visualize/{:entity}/load",
               get(event::visualize))
}

pub fn view_routes() -> Router {
    Router::new()
        .route("/definition/{:root}/{:namespace}/function/{:id}/view",
               get(function::view))
        .route("/definition/{:root}/{:namespace}/node/{:id}/view",
               get(node::view))
}


pub fn post_routes() -> Router {
    Router::new()
        .route("/definitions/compile",
               post(root::compile))

}
