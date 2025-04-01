use serde_derive::{Deserialize, Serialize};
use tabled::{Style, Table, Tabled};

use aws::{sfn, lambda, appsync};
use aws::Env;
use kit as u;
use std::collections::HashMap;
use compiler::{Topology, TopologyKind};

#[derive(Tabled, Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    namespace: String,
    sandbox: String,
    version: String,
    frozen: String,
    updated_at: String,
}

async fn get_graphql_api_arn(env: &Env, name: &str) -> Option<String> {
    let client = appsync::make_client(env).await;
    let api = appsync::find_api(&client, name).await;
    match api {
        Some(ap) => {
            Some(ap.arn)
        }
        None => None,
    }
}

async fn lookup_tags(env: &Env, kind: &TopologyKind, name: &str) -> HashMap<String, String> {
    match kind {
        TopologyKind::StepFunction => {
            let client = sfn::make_client(env).await;
            let states_arn = env.sfn_arn(&name);
            sfn::list_tags(&client, &states_arn).await.unwrap()
        },
        TopologyKind::Function => {
            let client = lambda::make_client(env).await;
            let lambda_arn = env.lambda_arn(&name);
            lambda::list_tags(client, &lambda_arn).await.unwrap()
        },
        TopologyKind::Graphql => {
            let client = appsync::make_client(env).await;
            let maybe_api_arn = get_graphql_api_arn(env, &name).await;
            if let Some(api_arn) = maybe_api_arn {
                appsync::list_tags(&client, &api_arn).await.unwrap()
            } else {
                HashMap::new()
            }
        },
        TopologyKind::Evented => {
            HashMap::new()
        }
    }
}

pub fn render(s: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    u::stencil(s, table)
}

pub async fn list(
    env: &Env,
    sandbox: &str,
    topologies: &HashMap<String, Topology>,
    format: &str
) {
    let mut rows: Vec<Record> = vec![];
    for (_, node) in topologies {
        let name = render(&node.fqn, sandbox);
        let tags = lookup_tags(env, &node.kind, &name).await;
        let namespace = u::safe_unwrap(tags.get("namespace"));
        if !&namespace.is_empty() {
            let version = u::safe_unwrap(tags.get("version"));
            if version != "0.0.1" {
                let row = Record {
                    namespace: namespace,
                    sandbox: u::safe_unwrap(tags.get("sandbox")),
                    version: version,
                    frozen: u::safe_unwrap(tags.get("freeze")),
                    updated_at: u::safe_unwrap(tags.get("updated_at")),
                };
                rows.push(row)
            }
        }
    }
    match format {
        "table" => {
            let table = Table::new(rows).with(Style::psql()).to_string();
            println!("{}", table);
        }
        "json" => {
            let s = u::pretty_json(rows);
            println!("{}", &s);
        }
        _ => (),
    }
}
