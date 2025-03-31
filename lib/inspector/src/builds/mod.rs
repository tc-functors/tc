mod layers;
mod images;
mod page;

use axum::{
    routing::{get, post},
    Router,
};

pub fn page_routes() -> Router {
    Router::new()
        .route("/builds", get(page::index))
        .route("/builds/list/{{entity}}", get(page::list))

}

pub fn list_routes() -> Router {
    Router::new()
        .route("/builds/hx/list/layers", get(layers::list))
}

pub fn post_routes() -> Router {
    Router::new()
        .route("/builds/sync", post(layers::sync))
}
