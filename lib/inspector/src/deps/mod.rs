use askama::Template;
use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod layers;

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
#[template(path = "deps/index.html")]
struct IndexTemplate {
    entity: String,
    context: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: String::from("layers"),
        context: String::from("deps"),
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/deps", get(index_page))
        .route("/hx/deps/list/layers", get(layers::list))
        .route("/hx/deps/sync", post(layers::sync))
}
