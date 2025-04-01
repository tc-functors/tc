mod changelog;
mod page;

use axum::{
    routing::{get},
    Router,
};


pub fn page_routes() -> Router {
    Router::new()
        .route("/releases", get(page::index))
        .route("/releases/list/{:entity}", get(page::list))
}

pub fn list_routes() -> Router {
    Router::new()
        .route("/hx/releases/list/changelog", get(changelog::list))
}
