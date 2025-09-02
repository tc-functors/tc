use authorizer::Auth;
use kit as u;
mod aws;
mod versions;

use composer::{
    Topology,
    TopologyKind,
};
use serde_derive::Serialize;
use std::collections::HashMap;
use tabled::{
    Table,
    settings::Style,
};
pub use versions::Record;

pub fn pretty_print(records: Vec<Record>, format: &str) {
    match format {
        "table" => {
            let table = Table::new(records).with(Style::psql()).to_string();
            println!("{}", table);
        }
        "json" => {
            let s = u::pretty_json(records);
            println!("{}", &s);
        }
        _ => (),
    }
}

pub async fn snapshot_profiles(dir: &str, sandbox: &str, profiles: Vec<String>) {
    let topologies = composer::compose_root(dir, false);
    versions::find_by_profiles(sandbox, profiles, topologies).await;
}

pub async fn snapshot(auth: &Auth, dir: &str, sandbox: &str) -> Vec<Record> {
    let topologies = composer::compose_root(dir, false);
    versions::find(auth, sandbox, topologies).await
}

pub async fn find_version(auth: &Auth, fqn: &str, kind: &TopologyKind) -> Option<String> {
    versions::find_version(auth, fqn, kind).await
}

#[derive(Debug, Clone, Serialize)]
struct Manifest {
    time: String,
    env: String,
    sandbox: String,
    namespace: String,
    version: String,
    tc_version: String,
    updated_at: String,
    changelog: Vec<String>,
    topology: Option<Topology>,
}

fn find_changelog(namespace: &str, version: &str) -> Vec<String> {
    if !version.is_empty() {
        u::split_lines(&tagger::changelogs_since_last(&namespace, &version))
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    }
}

async fn save_manifest(auth: &Auth, uri: &str, payload: &str, target_profile: Option<String>) {
    let auth = match target_profile {
        Some(p) => &init_auth(&p).await,
        None => auth,
    };

    let (bucket, key) = aws::s3::parts_of(uri);
    let client = aws::s3::make_client(auth).await;
    println!("Saving manifest to {}", uri);
    let _ = aws::s3::put_str(&client, &bucket, &key, payload).await;
}

async fn init_auth(target_profile: &str) -> Auth {
    let config = composer::config(&u::pwd());
    match std::env::var("TC_ASSUME_ROLE") {
        Ok(_) => {
            let role = config.ci.roles.get(target_profile).cloned();
            Auth::new(Some(target_profile.to_string()), role).await
        }
        Err(_) => Auth::new(Some(target_profile.to_string()), None).await,
    }
}

pub async fn generate_manifest(
    auth: &Auth,
    dir: &str,
    sandbox: &str,
    save: Option<String>,
    target_profile: Option<String>,
) {
    let topologies = composer::compose_root(dir, false);
    let versions = versions::find(auth, sandbox, topologies.clone()).await;
    let mut xs: HashMap<String, Manifest> = HashMap::new();

    for v in versions {
        let m = Manifest {
            time: u::utc_now(),
            version: v.version.clone(),
            namespace: v.namespace.clone(),
            env: auth.name.clone(),
            sandbox: v.sandbox,
            tc_version: v.tc_version,
            updated_at: v.updated_at,
            changelog: find_changelog(&v.namespace, &v.version),
            topology: topologies.get(&v.namespace).cloned(),
        };
        xs.insert(v.namespace.clone(), m);
    }

    let s = serde_json::to_string_pretty(&xs).unwrap();

    if let Some(uri) = save {
        save_manifest(auth, &uri, &s, target_profile).await
    } else {
        println!("{}", &s);
    }
}
