use axum::{
    extract::{DefaultBodyLimit},
    routing::{get, post},
    Router,
};

use memory_serve::{MemoryServe, load_assets};

mod page;
mod fragment;

pub async fn init() {

    let addr = "0.0.0.0:8000";

    let memory_router = MemoryServe::new(load_assets!("assets"))
        .into_router();

    let app = Router::new()
        .route("/", get(page::functors))
        .route("/functors", get(page::functors))
        .route("/functors/search", post(fragment::search_functors))
        .route("/functors/list", get(fragment::list_functors))
        .route("/functor/{:id}", get(page::get_functor))
        .route("/manifests", get(page::manifests))
        .route("/manifests/list", get(fragment::list_manifests))
        .route("/manifests/search", post(fragment::search_manifests))
        .route("/flow", get(page::flow))
        .route("/audit", get(page::audit))
        .route("/settings", get(page::settings))
        .route("/c4", get(page::c4))
        .route("/topology/{:id}", get(page::get_topology))
        .route("/functions/{:id}", get(page::get_topology))
        .merge(memory_router)
        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
