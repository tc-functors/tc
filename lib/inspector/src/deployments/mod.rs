mod page;
mod diff;


use axum::{
    routing::{get},
    Router,
};


pub fn page_routes() -> Router {
    Router::new()
        .route("/deployments", get(page::deployments))
}

pub fn list_routes() -> Router {
    Router::new()
        .route("/hx/deployments/list", get(diff::list))
}

pub fn diff_routes() -> Router {
    Router::new()
        .route("/hx/deployments/diff", get(diff::list))
}
