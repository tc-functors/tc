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
mod store;

pub async fn init() {

    let addr = "0.0.0.0:8000";

    let app = Router::new()
        .route("/"                    , get(page::definitions))
        .route("/definitions"         , get(page::definitions))
        .route("/builds"              , get(page::builds))
        .route("/builds/sync"         , post(builds::layers::sync))
        .route("/builds/layers/list"  , get(builds::layers::list))

        .route("/deployments"         , get(page::deployments))
        .route("/deployments/diff"    , get(deployments::diff::list))
        .route("/deployments/audit"   , post(deployments::audit::list))
        .route("/deployments/search"  , post(deployments::search::list))

        .route("/releases"            , get(page::releases))
        .route("/releases/changelog"  , get(releases::changelog::list))
        .route("/releases/snapshot"   , get(releases::snapshot::list))
        .route("/releases/nodes"      , get(releases::nodes::list_all))


        .route("/definition/{:root}",
               get(page::view_root_definition))
        .route("/definition/{:root}/{:entity}/{:id}",
               get(page::view_definition))
        .route("/definition/{:root}/{:namespace}/{:entity}/{:id}",
               get(page::view_entity_definition))

        .route("/definitions/{:entity}/all",
               get(page::list_all_definitions))
        .route("/definitions/{:root}/{:entity}",
               get(page::list_root_definitions))
        .route("/definitions/{:root}/{:namespace}/{:entity}",
               get(page::list_ns_definitions))

        // fragments root

        .route("/definitions/list",
               get(definitions::root::list_all))
        .route("/definitions/compile",
               post(definitions::root::compile))

        .route("/definitions/graph",
               post(definitions::root::generate_graph))

        .route("/definition/{:root}/{:namespace}/function/{:id}/view",
               get(definitions::function::view))
        .route("/definition/{:root}/{:namespace}/node/{:id}/view",
               get(definitions::node::view))


        // fragments list-all

        .route("/definitions/all/all/functions/list",
               get(definitions::function::list_all))

        .route("/definitions/all/all/nodes/list",
               get(definitions::node::list_all))

        .route("/definitions/all/all/events/list",
               get(definitions::event::list_all))

        .route("/definitions/all/all/routes/list",
               get(definitions::route::list_all))

        .route("/definitions/all/all/mutations/list",
               get(definitions::mutation::list_all))

        // fragments  list
        .route("/definitions/{:root}/{:namespace}/functions/list",
               get(definitions::function::list))
        .route("/definitions/{:root}/{:namespace}/nodes/list",
               get(definitions::node::list))
        .route("/definitions/{:root}/{:namespace}/events/list",
               get(definitions::event::list))
        .route("/definitions/{:root}/{:namespace}/mutations/list",
               get(definitions::mutation::list))
        .route("/definitions/{:root}/{:namespace}/routes/list",
               get(definitions::route::list))

        .layer(DefaultBodyLimit::disable())
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to start listener!");

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
