use axum::{
    extract::{DefaultBodyLimit},
    routing::{get, post},
    Router,
};

mod page;
mod fragment;

pub async fn init() {

    let addr = "0.0.0.0:8000";

    let app = Router::new()
        .route("/", get(page::functors))
        .route("/functors", get(page::functors))
        .route("/functors/search", post(fragment::search_functors))
        .route("/functors/list", get(fragment::list_functors))
        .route("/nodes/{:id}", get(page::nodes))
        .route("/nodes/list/{:id}", get(fragment::get_nodes))
        .route("/functions/{:id}", get(page::functions))
        .route("/functions/list/{:id}", get(fragment::get_functions))
        .route("/manifests", get(page::manifests))
        .route("/manifests/list", get(fragment::list_manifests))
        .route("/manifests/search", post(fragment::search_manifests))
        .route("/flow", get(page::flow))
        .route("/audit", get(page::audit))
        .route("/settings", get(page::settings))
        .route("/c4", get(page::c4))
        .route("/topology/{:id}", get(page::get_topology))
        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
