use axum::Router;

mod counter;
mod functor;
mod llm;
mod index;
mod overview;
mod store;

pub use store::Store;
use tracing_subscriber::{
    filter::{
        LevelFilter,
        Targets,
    },
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

pub async fn init(port: Option<String>) {
    let port = match port {
        Some(p) => p,
        None => String::from("8080"),
    };
    let addr = format!("0.0.0.0:{}", &port);
    println!("Composing topologies...");
    let topologies = composer::compose_root(&kit::pwd(), true);
    let store = Store::new().await;
    println!("Loading Graph database...");
    let _ = store.load(topologies).await;

    let filter = Targets::new()
        .with_target("tc-inspector", tracing::Level::DEBUG)
        .with_default(tracing::Level::DEBUG)
        .with_target("sqlx", LevelFilter::OFF);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let app = Router::new()
        .merge(index::page_routes(&store))
        .merge(functor::sidebar_routes(&store))
        .merge(functor::main_routes(&store))
        .merge(functor::list_routes(&store))
        .merge(functor::topology_routes(&store))
        .merge(functor::function_routes(&store))
        .merge(functor::mutation_routes(&store))
        .merge(overview::list_routes(&store))
        .merge(llm::render_routes(&store))
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    // let url = format!("http://localhost:{}", &port);
    // open::that(url).unwrap();
    axum::serve(listener, app).await.unwrap();
}
