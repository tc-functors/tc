mod page;
mod diff;
mod version;
mod functor;

use axum::{
    routing::{get},
    Router,
};


pub fn page_routes() -> Router {
    Router::new()
        .route("/deployments", get(page::index))
        .route("/deployments/list/{:entity}", get(page::list))
}

pub fn view_routes() -> Router {
    Router::new()
        .route("/hx/deployments/diff", get(diff::view))
        .route("/hx/deployments/versions", get(version::view))
        .route("/hx/deployments/functors", get(functor::view))
}

pub fn tab_routes() -> Router {
    Router::new()
        .route("/hx/deployments/tab/diff", get(diff::tab))
        .route("/hx/deployments/tab/versions", get(version::tab))
        .route("/hx/deployments/tab/functors", get(functor::tab))
}
