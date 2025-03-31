use axum::{
    extract::{DefaultBodyLimit},
    Router,
};

mod deployments;
mod definitions;
mod releases;
mod builds;
mod cache;

pub async fn init() {

    let addr = "0.0.0.0:8000";

    let app = Router::new()
        .merge(definitions::page_routes())
        .merge(definitions::list_routes())
        .merge(definitions::visualize_routes())
        .merge(definitions::view_routes())

        .merge(builds::page_routes())
        .merge(builds::list_routes())
        .merge(builds::post_routes())

        .merge(deployments::page_routes())
        .merge(deployments::list_routes())
        .merge(deployments::diff_routes())

        .merge(releases::page_routes())
        .merge(releases::list_routes())

        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
