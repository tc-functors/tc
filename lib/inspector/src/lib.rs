use axum::{
    extract::{DefaultBodyLimit},
    routing::{get, post},
    Router,
};

mod deployments;
mod definitions;
mod releases;
mod page;

pub async fn init() {

    let addr = "0.0.0.0:8000";

    let app = Router::new()
        .route("/"                                   , get(page::definitions))
        .route("/definitions"                        , get(page::definitions))
        .route("/definitions/search"                 , post(definitions::functors::list))
        .route("/definitions/list"                   , get(definitions::functors::list))
        .route("/definitions/compile"                , post(definitions::functors::compile))
        .route("/definitions/show-action-form"       , post(definitions::show_action_form))
        .route("/definitions/nodes/{:id}"            , get(page::nodes))
        .route("/definitions/nodes/list/{:id}"       , get(definitions::nodes::list))
        .route("/definitions/functions/{:id}"        , get(page::functions))
        .route("/definitions/functions/list/{:id}"   , get(definitions::functions::list))
        .route("/definitions/functions/events/{:id}" , get(page::events))
        .route("/definitions/events/list/{:id}"      , get(definitions::events::list))
        .route("/definitions/mutations/{:id}"        , get(page::mutations))
        .route("/definitions/mutations/list/{:id}"   , get(definitions::mutations::list))
        .route("/definitions/routes/{:id}"           , get(page::routes))
        .route("/definitions/routes/list/{:id}"      , get(definitions::routes::list))
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
