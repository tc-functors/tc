use askama::Template;
use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod versions;
mod functors;


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
#[template(path = "diffs/index.html")]
struct IndexTemplate {
    entity: String,
    context: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: String::from("default"),
        context: String::from("diffs"),
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/diffs",
               get(index_page))
        .route("/diffs/functors",
               get(functors::view))
        .route("/diffs/versions",
               get(versions::view))
}
