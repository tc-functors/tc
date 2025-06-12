use crate::cache;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{
        Html,
        IntoResponse,
        Response,
    },
};

async fn build_mermaid_str() -> Vec<String> {
    let events = cache::find_all_events().await;
    let mut xs: Vec<String> = vec![];
    let roots = cache::find_root_namespaces().await;

    for (_, event) in events {
        for t in event.targets {
            let producer = t.producer_ns;
            let consumer = t.consumer_ns;

            let target_name = &t
                .name
                .replace("{{namespace}}_", "")
                .replace("{{namespace}}-", "")
                .replace("_{{sandbox}}", "")
                .replace("-{{sandbox}}", "");

            if roots.contains(&consumer) && roots.contains(&producer) {
                let c = kit::split_first(&consumer, "-");
                let x = format!("{}->>{}: {}", producer, &c, &event.name);
                xs.push(x);
                let note = format!("note left of {}: λ {}", &c, target_name);
                xs.push(note);
            }
        }
    }
    xs
}

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
#[template(path = "overview/diagram.html")]
struct SequenceTemplate {
    items: Vec<String>,
}

pub async fn sequence() -> impl IntoResponse {
    let xs = build_mermaid_str().await;

    let temp = SequenceTemplate { items: xs };
    Html(temp.render().unwrap())
}
