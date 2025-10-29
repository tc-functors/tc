use kit as u;
use kit::*;
use provider::Auth;
mod manifest;
pub mod pipeline;

use compiler::TopologyKind;
use composer::Topology;
use std::collections::HashMap;
use configurator::Config;
pub use manifest::Manifest;
use provider::aws;
use tabled::{
    Table,
    builder::Builder,
    settings::Style,
};
use serde_derive::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub namespace: String,
    pub versions: HashMap<String, String>
}

pub async fn snapshot_profiles_json(topologies: &HashMap<String, Topology>, sandbox: &str, profiles: Vec<String>) -> Vec<Record> {
    let mut xs: Vec<Record> = vec![];
    for (_, node) in topologies {
        let name = manifest::render(&node.fqn, sandbox);

        let mut h: HashMap<String, String> = HashMap::new();

        for profile in &profiles {
            let auth = Auth::new(Some(s!(profile)), None).await;
            let tags = manifest::lookup_tags(&auth, &node.kind, &name).await;
            let version = u::safe_unwrap(tags.get("version"));
            h.insert(profile.to_string(), version);
        }
        xs.push(Record { namespace: node.namespace.clone(), versions: h })
    }
    xs
}


pub async fn snapshot_profiles(dir: &str, sandbox: &str, profiles: Vec<String>) {
    let topologies = composer::compose_root(dir, false);
    let mut builder = Builder::default();

    let mut cols: Vec<String> = vec![];
    cols.push(s!("Topology"));
    cols.extend(profiles.clone());

    builder.push_record(cols);

    for (_, node) in topologies {
        let mut row: Vec<String> = vec![];
        let name = manifest::render(&node.fqn, sandbox);

        row.push(s!(&node.namespace));

        for profile in &profiles {
            let auth = Auth::new(Some(s!(profile)), None).await;
            let tags = manifest::lookup_tags(&auth, &node.kind, &name).await;
            let version = u::safe_unwrap(tags.get("version"));
            row.push(version);
        }
        builder.push_record(row);
    }

    let mut table = builder.build();
    println!("{}", table.with(Style::psql()));
}

pub async fn find_version(auth: &Auth, fqn: &str, kind: &TopologyKind) -> Option<String> {
    let tags = manifest::lookup_tags(auth, kind, fqn).await;
    let namespace = u::safe_unwrap(tags.get("namespace"));
    if !&namespace.is_empty() {
        let version = u::safe_unwrap(tags.get("version"));
        if version != "0.0.1" {
            Some(version)
        } else {
            None
        }
    } else {
        None
    }
}

pub async fn snapshot_topologies(auth: &Auth, topologies: &HashMap<String, Topology>, sandbox: &str, gen_changelog: bool) -> Vec<Manifest> {
    let mut rows: Vec<Manifest> = vec![];
    for (_, node) in topologies {
        let row = Manifest::new(auth, sandbox, &node, gen_changelog).await;
        rows.push(row)
    }
    rows.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    rows.reverse();
    rows
}


pub async fn snapshot(auth: &Auth, dir: &str, sandbox: &str, gen_changelog: bool) -> Vec<Manifest> {
    let topologies = match std::env::var("TC_SNAPSHOT_BREAKOUT") {
        Ok(_) => composer::compose_root(dir, true),
        Err(_) => composer::compose_root(dir, false),
    };
    u::sh("git fetch --tags", dir);
    let mut rows: Vec<Manifest> = vec![];
    for (_, node) in topologies {
        let row = Manifest::new(auth, sandbox, &node, gen_changelog).await;
        rows.push(row)
    }
    rows.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    rows.reverse();
    rows
}

pub async fn show(name: &str) -> Vec<Manifest> {
    let cfg = Config::new();

    let maybe_bucket = cfg.snapshotter.bucket;
    let maybe_prefix = cfg.snapshotter.prefix;
    let maybe_target_profile = cfg.snapshotter.profile;

    if let (Some(bucket), Some(prefix), Some(profile)) =
        (maybe_bucket, maybe_prefix, maybe_target_profile)
    {
        let auth = &init_auth(&profile).await;
        let client = aws::s3::make_client(auth).await;
        let key = format!("{}/{}.json", &prefix, name);
        let s = aws::s3::get_str(&client, &bucket, &key).await;
        let xs: Vec<Manifest> = serde_json::from_str(&s).unwrap();
        xs
    } else {
        println!("No bucket configured");
        vec![]
    }
}

pub async fn list(_sandbox: &str) -> Vec<String> {
    let cfg = Config::new();

    let maybe_bucket = cfg.snapshotter.bucket;
    let maybe_prefix = cfg.snapshotter.prefix;
    let maybe_target_profile = cfg.snapshotter.profile;

    if let (Some(bucket), Some(prefix), Some(profile)) =
        (maybe_bucket, maybe_prefix, maybe_target_profile)
    {
        let auth = &init_auth(&profile).await;
        let client = aws::s3::make_client(auth).await;
        let keys = aws::s3::list_keys(&client, &bucket, &prefix).await;
        let mut xs: Vec<String> = vec![];
        for key in keys {
            let part = u::split_last(&key, &format!("{}/", &prefix));
            let name = u::split_first(&part, ".");
            xs.push(name)
        }
        xs.sort();
        xs.reverse();
        xs
    } else {
        tracing::debug!("No snapshot bucket configured. Skipping save");
        vec![]
    }
}

pub async fn save(auth: &Auth, payload: &str, env: &str, sandbox: &str) {
    let cfg = Config::new();

    let maybe_bucket = cfg.snapshotter.bucket;
    let maybe_prefix = cfg.snapshotter.prefix;
    let maybe_target_profile = cfg.snapshotter.profile;

    if let (Some(bucket), Some(prefix)) = (maybe_bucket, maybe_prefix) {
        let auth = match maybe_target_profile {
            Some(p) => &init_auth(&p).await,
            None => auth,
        };
        let key = format!("{}/{}/{}/{}.json", prefix, env, sandbox, u::ymd());
        let client = aws::s3::make_client(auth).await;
        tracing::debug!("Saving manifest to s3://{}/{}", &bucket, &key);
        let _ = aws::s3::put_str(&client, &bucket, &key, payload).await;
    } else {
        tracing::debug!("No snapshot bucket configured. Skipping save");
    }
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

pub fn pretty_print(
    records: &Vec<Manifest>,
    format: &str,
    env: Option<String>,
    sandbox: Option<String>,
) {
    match format {
        "table" => {
            let table = Table::new(records).with(Style::psql()).to_string();
            println!("{}", table);
        }
        "json" => {
            let s = u::pretty_json(records);
            println!("{}", &s);
        }
        "pipeline-config" | "pipeline" => {
            let env = match env {
                Some(e) => e,
                None => panic!("Please provide --target-env"),
            };

            let sandbox = match sandbox {
                Some(e) => e,
                None => panic!("Please provide --target-sandbox"),
            };
            let s = pipeline::generate_config(records, &env, &sandbox);
            println!("{}", &s);
        }
        _ => (),
    }
}
