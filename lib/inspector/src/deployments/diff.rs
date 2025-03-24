use std::collections::HashMap;
use configurator::Config;
use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use aws::{sfn, Env};
use crate::store;

struct Record {
    namespace: String,
    fqn: String,
}

async fn list_cached_records() -> Vec<Record> {
    let items = cache::list();
    let mut xs: Vec<Record> = vec![];
    for item in items {
        let key = cache::make_key(&item.namespace, &item.env, &item.sandbox);
        let maybe_topology = store::read_topology(&key).await;
        if let Some(topology) = maybe_topology {
            xs.push(Record {
                namespace: topology.namespace,
                fqn: topology.fqn
            });
        }
    }
    xs
}


async fn build(envs: Vec<String>, _sandbox: &str) -> HashMap<String, HashMap<String, String>> {
    // hashmap of namespace -> env -> version
    let records = list_cached_records().await;
    let mut h: HashMap<String, HashMap<String, String>> = HashMap::new();
    for r in records {
        let mut v: HashMap<String, String> = HashMap::new();
        for e in &envs {
            let env = Env::new(&e, None, Config::new(None, &e));
            let client = sfn::make_client(&env).await;
            let arn = &env.sfn_arn(&r.fqn);
            println!("{} - {}", &e, &arn);
            let version = sfn::get_tag(&client, arn, "version".to_string()).await;
            v.insert(e.to_string(), version);
        }
        h.insert(r.namespace, v);
    }
    tracing::debug!("{:?}", &h);
    h
}

#[derive(Template)]
#[template(path = "deployments/view/diff.html")]
struct FunctorsTemplate {
    envs: Vec<String>,
    items: HashMap<String, HashMap<String, String>>
}

pub async fn list() -> impl IntoResponse {
    let envs = vec![];
    let t = FunctorsTemplate {
        envs: envs.clone(),
        items: build(envs, "stable").await
    };
    Html(t.render().unwrap())
}
