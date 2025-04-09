use axum::{
    extract::{DefaultBodyLimit},
    Router,
};


mod definitions;
mod diagrams;
mod sandboxes;
mod releases;
mod deps;
mod diffs;
mod cache;

pub async fn init() {

    let addr = "0.0.0.0:8000";

    let app = Router::new()
        .merge(definitions::page_routes())
        .merge(definitions::list_routes())
        .merge(definitions::view_routes())
        .merge(definitions::post_routes())

        .merge(diagrams::routes())
        .merge(deps::routes())
        .merge(diffs::routes())
        .merge(sandboxes::routes())
        .merge(releases::routes())

        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
