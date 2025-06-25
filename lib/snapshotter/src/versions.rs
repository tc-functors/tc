use crate::aws::{
    appsync,
    lambda,
    sfn,
};
use authorizer::Auth;
use compiler::{Topology, TopologyKind};
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use tabled::{
    Tabled,
    builder::Builder,
    settings::Style,
};

#[derive(Tabled, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub namespace: String,
    pub sandbox: String,
    pub version: String,
    pub frozen: String,
    pub tc_version: String,
    pub updated_at: String,
}

async fn get_graphql_api_arn(auth: &Auth, name: &str) -> Option<String> {
    let client = appsync::make_client(auth).await;
    let api = appsync::find_api(&client, name).await;
    match api {
        Some(ap) => Some(ap.arn),
        None => None,
    }
}

async fn lookup_tags(auth: &Auth, kind: &TopologyKind, name: &str) -> HashMap<String, String> {
    match kind {
        TopologyKind::StepFunction => {
            let client = sfn::make_client(auth).await;
            let states_arn = auth.sfn_arn(&name);
            sfn::list_tags(&client, &states_arn).await.unwrap()
        }
        TopologyKind::Function => {
            let client = lambda::make_client(auth).await;
            let lambda_arn = auth.lambda_arn(&name);
            lambda::list_tags(client, &lambda_arn).await.unwrap()
        }
        TopologyKind::Graphql => {
            let client = appsync::make_client(auth).await;
            let maybe_api_arn = get_graphql_api_arn(auth, &name).await;
            if let Some(api_arn) = maybe_api_arn {
                appsync::list_tags(&client, &api_arn).await.unwrap()
            } else {
                HashMap::new()
            }
        }
        TopologyKind::Evented => HashMap::new(),
    }
}

fn render(s: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    u::stencil(s, table)
}

pub async fn find(auth: &Auth, sandbox: &str, topologies: HashMap<String, Topology>) -> Vec<Record> {
    let mut rows: Vec<Record> = vec![];
    for (_, node) in topologies {
        let name = render(&node.fqn, sandbox);
        let tags = lookup_tags(auth, &node.kind, &name).await;
        let namespace = u::safe_unwrap(tags.get("namespace"));
        if !&namespace.is_empty() {
            let version = u::safe_unwrap(tags.get("version"));
            if version != "0.0.1" {
                let row = Record {
                    namespace: namespace,
                    sandbox: u::safe_unwrap(tags.get("sandbox")),
                    version: version,
                    frozen: u::safe_unwrap(tags.get("freeze")),
                    tc_version: u::safe_unwrap(tags.get("tc_version")),
                    updated_at: u::safe_unwrap(tags.get("updated_at")),
                };
                rows.push(row)
            }
        }
    }
    rows
}

pub async fn find_by_profiles(sandbox: &str, profiles: Vec<String>, topologies: HashMap<String, Topology>) {

    let mut builder = Builder::default();

    let mut cols: Vec<String> = vec![];
    cols.push(s!("Topology"));
    cols.extend(profiles.clone());

    builder.push_record(cols);

    for (_, node) in topologies {
        let mut row: Vec<String> = vec![];
        let name = render(&node.fqn, sandbox);

        row.push(s!(&node.namespace));

        for profile in &profiles {
            let auth = Auth::new(Some(s!(profile)), None).await;
            let tags = lookup_tags(&auth, &node.kind, &name).await;
            let version = u::safe_unwrap(tags.get("version"));
            row.push(version);
        }
        builder.push_record(row);
    }

    let mut table = builder.build();
    println!("{}", table.with(Style::psql()));
}
