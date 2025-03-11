use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    Form,
};

use compiler::TopologyCount;

use serde::Deserialize;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Functor {
    id: String,
    namespace: String,
    env: String,
    sandbox: String,
    functions: usize,
    nodes: usize,
    events: usize,
    queues: usize,
    routes: usize,
    mutations: usize,
    states: usize,
    version: String
}

async fn find_functors() -> Vec<Functor> {
    let items = cache::list();
    let mut xs: Vec<Functor> = vec![];
    for item in items {
        let key = cache::make_key(&item.namespace, &item.env, &item.sandbox);
        let maybe_topology = cache::read_topology(&key).await;
        if let Some(topology) = maybe_topology {
            let t = TopologyCount::new(&topology);
            let f = Functor {
                id: key,
                namespace: item.namespace,
                env: item.env,
                sandbox: item.sandbox,
                functions: t.functions,
                nodes: t.nodes,
                events: t.events,
                queues: t.queues,
                routes: t.routes,
                states: t.states,
                mutations: t.mutations,
                version: String::from(&topology.version)
            };
            xs.push(f)
        }
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs.reverse();
    xs
}


#[derive(Template)]
#[template(path = "functors_list.html")]
struct FunctorsTemplate {
    items: Vec<Functor>
 }

#[derive(Deserialize, Debug)]
pub struct FunctorsInput {
    pub env: String,
    pub sandbox: String,
    pub kind: String
}

pub async fn search_functors(Form(payload): Form<FunctorsInput>) -> impl IntoResponse {
    let FunctorsInput { env, sandbox, kind, .. } = payload;

    let xs = find_functors().await;
    tracing::debug!("search {} - {}", env, sandbox);

    let functors = if &kind == "all" {
        xs
    } else {
        xs.into_iter().filter(|x| x.version != "0.0.1").collect()
    };

    let t = FunctorsTemplate {
        items: functors
    };
    Html(t.render().unwrap())
}


pub async fn list_functors() -> impl IntoResponse {
    let xs = find_functors().await;
    let functors: Vec<_> = xs.into_iter().filter(|x| x.version != "0.0.1").collect();
    let t = FunctorsTemplate {
        items: functors
    };
    Html(t.render().unwrap())
}

struct Manifest {
    namespace: String,
    prod: String,
    qa: String,
    staging: String,
    poc: String
}


#[derive(Template)]
#[template(path = "manifests_list.html")]
struct ManifestsTemplate {
    items: Vec<Manifest>
 }

pub async fn list_manifests() -> impl IntoResponse {
    let manifests = vec![];
    let t = ManifestsTemplate {
        items: manifests
    };
    Html(t.render().unwrap())
}

pub async fn search_manifests() -> impl IntoResponse {
    let manifests = vec![];
    let t = ManifestsTemplate {
        items: manifests
    };
    Html(t.render().unwrap())
}
