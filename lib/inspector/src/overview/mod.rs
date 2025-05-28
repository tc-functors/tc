mod functors;
mod events;
mod functions;
mod mutations;
mod nodes;
mod routes;
mod diagram;

use askama::Template;
use axum::{
    Router,
    http::StatusCode,
    response::{
        Html,
        IntoResponse,
        Response,
    },
    routing::{
        get,
    },
};

pub struct HtmlTemplate<T>(pub T);
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "overview/index.html")]
struct IndexTemplate {
    root: String,
    namespace: String,
}

pub async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        root: "overview".to_string(),
        namespace: "overview".to_string(),
    };
    HtmlTemplate(template)
}

pub fn page_routes() -> Router {
    Router::new()
        .route("/overview", get(index))
}

// fragments

pub fn list_routes() -> Router {
    Router::new()
        .route("/hx/overview/functors", get(functors::list))
        .route("/hx/overview/functions", get(functions::list))
        .route("/hx/overview/nodes", get(nodes::list))
        .route("/hx/overview/diagram", get(diagram::sequence))
        .route("/hx/overview/events", get(events::list))
        .route("/hx/overview/routes", get(routes::list))
        .route("/hx/overview/mutations", get(mutations::list))
}
