use crate::{
    cache,
    cache::Versions,
};
use askama::Template;
use axum::response::{
    Html,
    IntoResponse,
};
use compiler::Topology;
use configurator::Config;
use std::collections::HashMap;

async fn build(
    envs: Vec<String>,
    topologies: HashMap<String, Topology>,
    sandbox: &str,
) -> Versions {
    let mut h: Versions = HashMap::new();
    for (_, t) in topologies {
        let mut v: HashMap<String, String> = HashMap::new();
        for e in &envs {
            let env = provider::init(Some(e.to_string()), None, Config::new(None, &e)).await;
            let version = grokker::lookup_version(&env, &t.kind, &t.fqn, sandbox).await;
            if let Some(ver) = version {
                v.insert(e.to_string(), ver);
            }
        }
        h.insert(t.namespace, v);
    }
    tracing::debug!("{:?}", &h);
    h
}

#[derive(Template)]
#[template(path = "diffs/versions_list.html")]
struct VersionsTemplate {
    envs: Vec<String>,
    items: HashMap<String, HashMap<String, String>>,
}

pub async fn generate() -> impl IntoResponse {
    let envs = vec![
        String::from("qa"),
        String::from("staging"),
        String::from("prod-01"),
        String::from("prod"),
    ];
    let topologies = cache::find_all_topologies().await;
    let versions = match cache::find_versions().await {
        Some(v) => v,
        None => {
            let vers = build(envs.clone(), topologies, "stable").await;
            cache::save_versions(vers.clone()).await;
            vers
        }
    };

    let t = VersionsTemplate {
        envs: envs.clone(),
        items: versions,
    };
    Html(t.render().unwrap())
}

#[derive(Template)]
#[template(path = "diffs/versions.html")]
struct ViewTemplate {
    entity: String,
    context: String,
    envs: Vec<String>,
}

pub async fn view() -> impl IntoResponse {
    let temp = ViewTemplate {
        entity: String::from("versions"),
        context: String::from("diffs"),
        envs: vec![
            String::from("qa"),
            String::from("staging"),
            String::from("prod-01"),
            String::from("prod"),
        ],
    };
    Html(temp.render().unwrap())
}
