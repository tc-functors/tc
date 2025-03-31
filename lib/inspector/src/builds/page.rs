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
struct BuildsTemplate { context: String }

pub async fn builds() -> impl IntoResponse {
    let template = BuildsTemplate {
        context: "builds".to_string()
    };
    HtmlTemplate(template)
}
