use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{
        Html,
        IntoResponse,
        Response,
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
    entity: String,
    context: String,
}

pub async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        root: "root".to_string(),
        namespace: "root".to_string(),
        entity: "functors".to_string(),
        context: "overview".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "overview/list.html")]
struct ListTemplate {
    root: String,
    namespace: String,
    entity: String,
    context: String,
}

pub async fn list_root(Path((root, entity)): Path<(String, String)>) -> impl IntoResponse {
    HtmlTemplate(ListTemplate {
        root: root.clone(),
        namespace: root,
        entity: entity,
        context: String::from("overview"),
    })
}

pub async fn list_ns(
    Path((root, namespace, entity)): Path<(String, String, String)>,
) -> impl IntoResponse {
    HtmlTemplate(ListTemplate {
        root: root,
        namespace: namespace,
        entity: entity,
        context: String::from("overview"),
    })
}

pub async fn list_all(Path(entity): Path<String>) -> impl IntoResponse {
    HtmlTemplate(ListTemplate {
        root: String::from("all"),
        namespace: String::from("all"),
        entity: entity,
        context: String::from("overview"),
    })
}

#[derive(Template)]
#[template(path = "overview/view.html")]
struct ViewTemplate {
    id: String,
    root: String,
    namespace: String,
    entity: String,
    context: String,
}

pub async fn _view_namespace(Path(id): Path<String>) -> impl IntoResponse {
    HtmlTemplate(ViewTemplate {
        id: id.clone(),
        root: id.clone(),
        namespace: id,
        entity: String::from("functor"),
        context: String::from("overview"),
    })
}

pub async fn view_entity(
    Path((root, namespace, entity, id)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    HtmlTemplate(ViewTemplate {
        id: id,
        root: root,
        namespace: namespace,
        entity: entity,
        context: String::from("overview"),
    })
}

pub async fn view_root(Path(root): Path<String>) -> impl IntoResponse {
    HtmlTemplate(ViewTemplate {
        id: root.clone(),
        root: root.clone(),
        namespace: root,
        entity: String::from("node"),
        context: String::from("overview"),
    })
}
