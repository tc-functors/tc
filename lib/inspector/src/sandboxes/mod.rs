use askama::Template;
use axum::{
    routing::{get, post},
    extract::Path,
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod list;
mod create;
mod test;
mod clone;

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
#[template(path = "sandboxes/index.html")]
struct IndexTemplate {
    entity: String,
    context: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: String::from("default"),
        context: String::from("sandboxes"),
    })
}

pub async fn view_page(Path(entity): Path<String>) -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: entity,
        context: String::from("sandboxes"),
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/sandboxes",
               get(index_page))
        .route("/sandboxes/{:entity}",
               get(view_page))
}
