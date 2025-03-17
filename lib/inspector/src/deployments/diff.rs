use std::collections::HashMap;
use configurator::Config;
use askama::Template;
use axum::{
    response::{Html, IntoResponse},
    Form
};

use aws::{sfn, Env};
use serde_derive::Deserialize;

struct Record {
    namespace: String,
    fqn: String,
}

async fn list_cached_records() -> Vec<Record> {
    let items = cache::list();
    let mut xs: Vec<Record> = vec![];
    for item in items {
        let key = cache::make_key(&item.namespace, &item.env, &item.sandbox);
        let maybe_topology = cache::read_topology(&key).await;
        if let Some(topology) = maybe_topology {
            xs.push(Record {
                namespace: topology.namespace,
                fqn: topology.fqn
            });
        }
    }
    xs
}


async fn build(envs: Vec<String>, sandbox: &str) -> HashMap<String, HashMap<String, String>> {
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
#[template(path = "deployments/fragments/diff.html")]
struct FunctorsTemplate {
    envs: Vec<String>,
    items: HashMap<String, HashMap<String, String>>
}

async fn maybe_build(envs: Vec<String>, sandbox: &str) -> HashMap<String, HashMap<String, String>> {
    let key = format!("deployments.{}.versions", sandbox);
    if cache::has_key(&key) {
        tracing::info!("Found deployments cache: {}", key);
        let s = cache::read(&key);
        let t: HashMap<String, HashMap<String, String>> = serde_json::from_str(&s).unwrap();
        println!("{:?}", &t);
        t
    } else {
        let data = build(envs, sandbox).await;
        let s = serde_json::to_string(&data).unwrap();
        cache::write(&key, &s).await;
        data
    }
}

pub async fn list() -> impl IntoResponse {
    let envs = vec![String::from("qa"), String::from("poc"), String::from("prod"), String::from("prod-01"), String::from("staging")];
    let t = FunctorsTemplate {
        envs: envs.clone(),
        items: build(envs, "stable").await
    };
    Html(t.render().unwrap())
}

#[derive(Deserialize, Debug)]
pub struct SearchInput {
    pub profiles: String,
    pub sandbox: String,
}

pub async fn search(Form(payload): Form<SearchInput>) -> impl IntoResponse {
    let SearchInput { profiles, sandbox }  = payload;
    let envs: Vec<String> = profiles.split(",").collect::<Vec<_>>()
        .into_iter().map(|s| s.to_owned()).collect();

    let t = FunctorsTemplate {
        envs: envs.clone(),
        items: build(envs, &sandbox).await
    };
    Html(t.render().unwrap())
}
