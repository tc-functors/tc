use askama::Template;
use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod versions;
mod manifests;


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
    context: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        context: String::from("diffs"),
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/diffs",
               get(index_page))
        .route("/hx/diffs/manifests",
               get(manifests::generate))
        .route("/hx/diffs/versions",
               get(versions::generate))
}
