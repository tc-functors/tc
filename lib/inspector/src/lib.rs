use axum::{
    extract::{DefaultBodyLimit},
    routing::{get, post},
    Router,
};

mod deployments;
mod definitions;
mod releases;
mod builds;
mod page;

pub async fn init() {

    let addr = "0.0.0.0:8000";

    let app = Router::new()
        .route("/"                                   , get(page::definitions))
        .route("/definitions"                        , get(page::definitions))
        .route("/definitions/search"                 , post(definitions::functor::list))
        .route("/definitions/list"                   , get(definitions::functor::list))
        .route("/definitions/compile"                , post(definitions::functor::compile))

        .route("/definitions/{:entity}/{:id}"        , get(page::list_definitions))
        .route("/definition/{:entity}/{:id}"         , get(page::view_definition))

        .route("/definitions/list/nodes/{:id}"       , get(definitions::node::list))
        .route("/definitions/list/functions/{:id}"   , get(definitions::function::list))
        .route("/definitions/list/events/{:id}"      , get(definitions::event::list))
        .route("/definitions/list/mutations/{:id}"   , get(definitions::mutation::list))
        .route("/definitions/list/routes/{:id}"      , get(definitions::route::list))


        .route("/definition/view/node/{:id}"         , get(definitions::node::view))
        .route("/definition/view/function/{:id}"     , get(definitions::function::view))

        .route("/builds"                             , get(page::builds))
        .route("/builds/sync"                        , post(builds::layers::sync))
        .route("/builds/list/layers"                 , get(builds::layers::list))

        .route("/deployments"                        , get(page::deployments))
        .route("/deployments/diff"                   , get(deployments::diff::list))
        .route("/deployments/audit"                  , post(deployments::audit::list))
        .route("/deployments/search"                 , post(deployments::search::list))

        .route("/releases"                           , get(page::releases))
        .route("/releases/list"                      , get(releases::functors::list))
        .route("/releases/changelog"                 , get(releases::changelog::list))
        .route("/releases/snapshot"                  , get(releases::snapshot::list))
        .route("/topology/{:id}"                     , get(page::get_topology))
        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
