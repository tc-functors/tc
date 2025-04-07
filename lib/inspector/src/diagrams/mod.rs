use askama::Template;
use axum::{
    extract::Path,
    routing::{get, post},
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod sequence;
mod flow;

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
#[template(path = "diagrams/index.html")]
struct IndexTemplate {
    entity: String,
    context: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: String::from("flow"),
        context: String::from("diagrams"),
    })
}

pub async fn view_page(Path(entity): Path<String>) -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: entity,
        context: String::from("diagrams"),
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/diagrams",
               get(index_page))
        .route("/diagrams/{:entity}",
               get(view_page))
        .route("/hx/diagrams/sequence",
               get(sequence::generate))
        .route("/hx/diagrams/flow",
               get(flow::generate))
}
