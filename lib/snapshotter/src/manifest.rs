use crate::aws::{
    appsync,
    lambda,
    sfn,
};
use authorizer::Auth;
use composer::{
    Topology,
    TopologyKind,
};
use kit as u;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

use tabled::{
    Tabled,
};

async fn get_graphql_api_arn(auth: &Auth, name: &str) -> Option<String> {
    let client = appsync::make_client(auth).await;
    appsync::find_api(&client, name).await
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

#[derive(Tabled, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    #[tabled(skip)]
    pub dir: String,
    pub namespace: String,
    pub kind: String,
    pub sandbox: String,
    pub version: String,
    pub git_version: String,
    pub frozen: String,
    pub tc_version: String,
    #[tabled(skip)]
    pub changelog: Option<Vec<String>>,
    pub updated_at: String,
    pub updated_by: String,
}

impl Manifest {

    pub async fn new(auth: &Auth, sandbox: &str, topology: &Topology, gen_changelog: bool) -> Manifest {
        let Topology { kind, dir, fqn, .. } = topology;

        let name = render(&fqn, sandbox);
        let tags = lookup_tags(auth, &kind, &name).await;
        let namespace = u::safe_unwrap(tags.get("namespace"));
        let version = u::safe_unwrap(tags.get("version"));
        let name = if namespace.is_empty() {
            &topology.namespace
        } else {
            &namespace
        };

        let changelog = if gen_changelog {
            Some(find_changelog(&namespace, &version))
        } else {
            None
        };

        let maybe_rdir = dir.strip_prefix(&format!("{}/", u::root()));
        let dir = match maybe_rdir {
            Some(d) => d,
            None => dir
        };

        Manifest {
            dir: dir.to_string(),
            namespace: name.to_string(),
            sandbox: u::safe_unwrap(tags.get("sandbox")),
            kind: kind.to_str(),
            version: version,
            git_version: topology.version.to_string(),
            frozen: u::safe_unwrap(tags.get("freeze")),
            tc_version: u::safe_unwrap(tags.get("tc_version")),
            changelog: changelog,
            updated_at: u::safe_unwrap(tags.get("updated_at")),
            updated_by: u::safe_unwrap(tags.get("updated_by")),
        }
    }

}
