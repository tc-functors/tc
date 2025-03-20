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
    name: String,
}

pub async fn definitions() -> impl IntoResponse {
    let template = DefinitionsTemplate {
        name: "definitions".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "builds/index.html")]
struct BuildsTemplate { name: String }

pub async fn builds() -> impl IntoResponse {
    let template = BuildsTemplate {
        name: "builds".to_string()
    };
    HtmlTemplate(template)
}


#[derive(Template)]
#[template(path = "deployments/index.html")]
struct DeploymentsTemplate { name: String }

pub async fn deployments() -> impl IntoResponse {
    let template = DeploymentsTemplate {
        name: "deployments".to_string()
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "releases/index.html")]
struct ReleasesTemplate { name: String }

pub async fn releases() -> impl IntoResponse {
    let template = ReleasesTemplate {
        name: "releases".to_string()
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "definitions/list.html")]
struct ListTemplate {
    id: String,
    entity: String,
    name: String,
}

pub async fn list_definitions(Path((entity, id)): Path<(String, String)>) -> impl IntoResponse {
    HtmlTemplate(ListTemplate {
        id: id,
        entity: entity,
        name: String::from("definitions"),
    })
}


#[derive(Template)]
#[template(path = "definitions/view.html")]
struct ViewTemplate {
    id: String,
    entity: String,
    name: String,
}

pub async fn view_definition(Path((entity, id)): Path<(String, String)>) -> impl IntoResponse {
    HtmlTemplate(ViewTemplate {
        id: id,
        entity: entity,
        name: String::from("definitions"),
    })
}


#[derive(Template)]
#[template(path = "definitions/topology.html")]
struct TopologyTemplate {
    id: String,
    name: String,
    topology: String
}

pub async fn get_topology(Path(id): Path<String>) -> impl IntoResponse {
    let maybe_topology = cache::read_topology(&id).await;
    let t = match maybe_topology {
        Some(topology) => topology.to_str(),
        None => String::from("Topology not found")
    };

    let template = TopologyTemplate {
        id: id,
        name: "topology".to_string(),
        topology: t
    };
    HtmlTemplate(template)
}
