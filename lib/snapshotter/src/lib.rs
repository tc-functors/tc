use authorizer::Auth;
use kit as u;
use kit::*;
mod aws;
mod manifest;
mod pipeline;

use composer::{
    TopologyKind,
};
use tabled::{
    Table,
    builder::Builder,
    settings::Style,
};

pub use manifest::Manifest;

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

pub async fn snapshot(auth: &Auth, dir: &str, sandbox: &str, gen_changelog: bool) -> Vec<Manifest> {
    let topologies = composer::compose_root(dir, false);
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

pub fn load(s: &str) -> Vec<Manifest> {
    let xs: Vec<Manifest> = serde_json::from_str(s).unwrap();
    xs
}

pub async fn save(auth: &Auth, uri: &str, payload: &str, target_profile: Option<String>) {
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

pub fn pretty_print(records: &Vec<Manifest>, format: &str, env: Option<String>, sandbox: Option<String>) {
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
                None => panic!("Please provide --target-env")
            };

            let sandbox = match sandbox {
                Some(e) => e,
                None => panic!("Please provide --target-sandbox")
            };
            let s = pipeline::generate_config(records, &env, &sandbox);
            println!("{}", &s);
        }
        _ => (),
    }
}
