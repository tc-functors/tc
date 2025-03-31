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
#[template(path = "definitions/index.html")]
struct DefinitionsTemplate {
    context: String,
}

pub async fn definitions() -> impl IntoResponse {
    let template = DefinitionsTemplate {
        context: "definitions".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "definitions/visual.html")]
struct VisualTemplate {
    root: String,
    namespace: String,
    entity: String,
    context: String,
}

pub async fn visualize(Path(entity): Path<String>) -> impl IntoResponse {
    HtmlTemplate(VisualTemplate {
        root: String::from("all"),
        namespace: String::from("all"),
        entity: entity,
        context: String::from("definitions"),
    })
}


#[derive(Template)]
#[template(path = "definitions/list.html")]
struct ListTemplate {
    root: String,
    namespace: String,
    entity: String,
    context: String,
}

pub async fn list_root_definitions(Path((root, entity)): Path<(String, String)>) -> impl IntoResponse {
    HtmlTemplate(ListTemplate {
        root: root.clone(),
        namespace: root,
        entity: entity,
        context: String::from("definitions"),
    })
}

pub async fn list_ns_definitions(Path((root, namespace, entity)): Path<(String, String, String)>) -> impl IntoResponse {
    HtmlTemplate(ListTemplate {
        root: root,
        namespace: namespace,
        entity: entity,
        context: String::from("definitions"),
    })
}

pub async fn list_all_definitions(Path(entity): Path<String>) -> impl IntoResponse {
    HtmlTemplate(ListTemplate {
        root: String::from("all"),
        namespace: String::from("all"),
        entity: entity,
        context: String::from("definitions"),
    })
}


#[derive(Template)]
#[template(path = "definitions/view.html")]
struct ViewTemplate {
    id: String,
    root: String,
    namespace: String,
    entity: String,
    context: String,
}

pub async fn view_definition(Path(id): Path<String>) -> impl IntoResponse {
    HtmlTemplate(ViewTemplate {
        id: id.clone(),
        root: id.clone(),
        namespace: id,
        entity: String::from("functor"),
        context: String::from("definitions"),
    })
}

pub async fn view_entity_definition(
    Path((root, namespace, entity, id)): Path<(String, String, String, String)>
) -> impl IntoResponse {
    HtmlTemplate(ViewTemplate {
        id: id,
        root: root,
        namespace: namespace,
        entity: entity,
        context: String::from("definitions"),
    })
}


pub async fn view_root_definition(Path(root): Path<String>) -> impl IntoResponse {
    HtmlTemplate(ViewTemplate {
        id: root.clone(),
        root: root.clone(),
        namespace: root,
        entity: String::from("node"),
        context: String::from("definitions"),
    })
}
