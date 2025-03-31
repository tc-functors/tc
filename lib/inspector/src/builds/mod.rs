mod layers;
mod images;
mod page;

use axum::{
    routing::{get, post},
    Router,
};

pub fn page_routes() -> Router {
    Router::new()
        .route("/builds", get(page::builds))

}

pub fn list_routes() -> Router {
    Router::new()
        .route("/builds/layers/list", get(layers::list))
}

pub fn post_routes() -> Router {
    Router::new()
        .route("/builds/sync", post(layers::sync))
}
