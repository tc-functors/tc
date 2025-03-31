mod changelog;
mod page;

use axum::{
    routing::{get},
    Router,
};


pub fn page_routes() -> Router {
    Router::new()
        .route("/releases", get(page::releases))
}

pub fn list_routes() -> Router {
    Router::new()
        .route("/releases/list/changelog", get(changelog::list))
}
