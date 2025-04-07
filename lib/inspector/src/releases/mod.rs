use askama::Template;
use axum::{
    extract::Path,
    routing::{get},
    Router,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

mod changelog;
mod timeline;
mod new;
mod current;

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
#[template(path = "releases/index.html")]
struct IndexTemplate {
    entity: String,
    context: String,
}

pub async fn index_page() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate {
        entity: String::from("default"),
        context: String::from("releases"),
    })
}

pub fn routes() -> Router {
    Router::new()
        .route("/releases",
               get(index_page))
        .route("/releases/timeline",
               get(timeline::view))
        .route("/releases/changelog",
               get(changelog::view))
        .route("/releases/new",
               get(new::view))
        .route("/releases/current",
               get(current::view))

}
