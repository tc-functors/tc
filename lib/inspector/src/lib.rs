use axum::{
    Router,
    extract::DefaultBodyLimit,
};

mod cache;
mod diffs;
mod functors;
mod overview;
mod releases;
mod sandboxes;
mod specs;

pub async fn init() {
    let addr = "0.0.0.0:8000";

    let app = Router::new()
        .merge(overview::page_routes())
        .merge(overview::list_routes())
        .merge(overview::view_routes())
        .merge(overview::post_routes())
        .merge(functors::page_routes())
        .merge(functors::entity_routes())
        .merge(functors::functor_routes())
        .merge(functors::function_routes())
        .merge(functors::mutation_routes())
        .merge(diffs::routes())
        .merge(sandboxes::routes())
        .merge(releases::routes())
        .merge(specs::routes())
        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
