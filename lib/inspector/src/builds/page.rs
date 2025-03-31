use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
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
#[template(path = "builds/index.html")]
struct IndexTemplate { context: String }

pub async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        context: "builds".to_string()
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "builds/list.html")]
struct ListTemplate {
    context: String,
    entity: String
}

pub async fn list(Path(entity): Path<String>) -> impl IntoResponse {
    let template = ListTemplate {
        context: "builds".to_string(),
        entity: entity
    };
    HtmlTemplate(template)
}
