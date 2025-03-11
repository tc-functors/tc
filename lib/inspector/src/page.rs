use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use compiler::Topology;

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
#[template(path = "functors.html")]
struct FunctorsTemplate {
    name: String,
}

pub async fn functors() -> impl IntoResponse {
    let template = FunctorsTemplate {
        name: "functors".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "manifests.html")]
struct ManifestsTemplate { name: String }

pub async fn manifests() -> impl IntoResponse {
    let template = ManifestsTemplate { name: "manifests".to_string() };
    HtmlTemplate(template)
}


#[derive(Template)]
#[template(path = "functor.html")]
struct FunctorTemplate {
    id: String,
    name: String,
    topology: Topology
}

pub async fn get_functor(Path(id): Path<String>) -> impl IntoResponse {
    let maybe_topology = cache::read_topology(&id).await;

    if let Some(t) = maybe_topology {
        HtmlTemplate(FunctorTemplate {
            id: id,
            name: String::from("functor"),
            topology: t
        })
    } else {
        panic!("error")
    }
}

#[derive(Template)]
#[template(path = "topology.html")]
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

#[derive(Template)]
#[template(path = "flow.html")]
struct FlowTemplate {
    name: String
}

pub async fn flow() -> impl IntoResponse {
    let template = FlowTemplate { name: "flow".to_string() };
    HtmlTemplate(template)
}


#[derive(Template)]
#[template(path = "audit.html")]
struct AuditTemplate {
    name: String
}

pub async fn audit() -> impl IntoResponse {
    let template = AuditTemplate { name: "audit".to_string() };
    HtmlTemplate(template)
}


#[derive(Template)]
#[template(path = "c4.html")]
struct C4Template {
    name: String
}

pub async fn c4() -> impl IntoResponse {
    let template = C4Template { name: "c4".to_string() };
    HtmlTemplate(template)
}


#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate {
    name: String
}

pub async fn settings() -> impl IntoResponse {
    let template = SettingsTemplate { name: "settings".to_string() };
    HtmlTemplate(template)
}
