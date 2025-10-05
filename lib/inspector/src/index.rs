use crate::{
    Store,
    counter,
};
use askama::Template;
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{
        Html,
        IntoResponse,
        Response,
    },
    routing::get,
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
struct OverviewTemplate {
    scope: String,
    functions: usize,
    events: usize,
    routes: usize,
    queues: usize,
    channels: usize,
    mutations: usize,
    states: usize,
    pages: usize,
}

#[derive(Template)]
#[template(path = "functor/index.html")]
struct FunctorsTemplate {
    scope: String,
    root: String,
    namespace: String,
}

pub async fn index(State(store): State<Store>) -> impl IntoResponse {
    let topologies = store.list_topologies().await;
    let counter = counter::count_all(&topologies).await;
    let template = OverviewTemplate {
        scope: "overview".to_string(),
        functions: counter.functions,
        events: counter.events,
        routes: counter.routes,
        queues: counter.queues,
        channels: counter.channels,
        mutations: counter.mutations,
        states: counter.states,
        pages: counter.pages,
    };
    HtmlTemplate(template)
}

pub async fn functors(State(_store): State<Store>) -> impl IntoResponse {
    let template = FunctorsTemplate {
        scope: "functors".to_string(),
        root: "functors".to_string(),
        namespace: "functors".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "llm/index.html")]
struct LlmTemplate {
    scope: String,
}

pub async fn llm(State(_store): State<Store>) -> impl IntoResponse {
    let template = LlmTemplate {
        scope: "llm".to_string(),
    };
    HtmlTemplate(template)
}

pub fn page_routes(store: &Store) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/overview", get(index))
        .route("/functors", get(functors))
        .route("/llm", get(llm))
        .with_state(store.clone())
}
