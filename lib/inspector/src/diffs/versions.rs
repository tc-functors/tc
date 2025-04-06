use std::collections::HashMap;
use configurator::Config;
use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use aws::Env;
use compiler::Topology;
use crate::cache::Versions;
use crate::cache;


async fn build(
    envs: Vec<String>,
    topologies: HashMap<String, Topology>,
    sandbox: &str
) -> Versions {

    let mut h: Versions = HashMap::new();
    for (_, t) in topologies {
        let mut v: HashMap<String, String> = HashMap::new();
        for e in &envs {
            let env = Env::new(&e, None, Config::new(None, &e));
            let version = lister::lookup_version(
                &env, &t.kind, &t.fqn, sandbox
            ).await;
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
#[template(path = "diffs/versions.html")]
struct VersionsTemplate {
    envs: Vec<String>,
    items: HashMap<String, HashMap<String, String>>
}

pub async fn generate() -> impl IntoResponse {
    let envs = vec![String::from("qa"),
                    String::from("staging"),
                    String::from("prod-01"),
                    String::from("prod")];
    let topologies = cache::find_all_topologies().await;
    let versions = match cache::find_versions().await  {
        Some(v) => v,
        None => build(envs.clone(), topologies, "stable").await
    };

    let t = VersionsTemplate {
        envs: envs.clone(),
        items: versions
    };
    Html(t.render().unwrap())
}
