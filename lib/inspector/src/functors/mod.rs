use askama::Template;
use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod loader;
mod flow;
mod definition;

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
#[template(path = "functors/index.html")]
struct IndexTemplate {
    entity: String,
    context: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: String::from("functors"),
        context: String::from("functors"),
    })
}


pub fn routes() -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/functors", get(index_page))
        .route("/hx/functor/{:name}", post(definition::view))
        .route("/hx/functors/load", post(loader::load))
        .route("/hx/functors/list", get(loader::list))
}
