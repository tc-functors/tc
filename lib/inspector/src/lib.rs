use axum::{
    Router,
    extract::DefaultBodyLimit,
};

mod cache;
mod functor;
mod overview;
mod list;

pub async fn init(port: Option<String>) {
    let port = match port {
        Some(p) => p,
        None => String::from("8000")
    };
    let addr = format!("0.0.0.0:{}", &port);

    println!("Loading current directory..");
    cache::init().await;

    let app = Router::new()
        .merge(list::routes())
        .merge(functor::page_routes())
        .merge(functor::list_routes())
        .merge(functor::topology_routes())
        .merge(functor::function_routes())
        .merge(functor::mutation_routes())
        .merge(overview::page_routes())
        .merge(overview::list_routes())
        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    // let url = format!("http://localhost:{}", &port);
    // open::that(url).unwrap();
    axum::serve(listener, app).await.unwrap();
 }
