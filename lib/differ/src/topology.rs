use compiler::{
    Topology,
    TopologyKind,
};
use kit as u;
use authorizer::Auth;
use crate::{
    aws::{
        appsync,
        lambda,
        sfn,
    },
};
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use tabled::Tabled;

#[derive(Tabled, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub namespace: String,
    pub sandbox: String,
    pub version: String,
    pub frozen: String,
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

pub async fn lookup_tags(auth: &Auth, kind: &TopologyKind, name: &str) -> HashMap<String, String> {
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

pub fn render(s: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    u::stencil(s, table)
}

pub async fn list(auth: &Auth, sandbox: &str, topologies: &HashMap<String, Topology>) -> Vec<Record> {
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
                    updated_at: u::safe_unwrap(tags.get("updated_at")),
                };
                rows.push(row)
            }
        }
    }
    rows
}
