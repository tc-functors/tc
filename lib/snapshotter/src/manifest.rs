use compiler::TopologyKind;
use composer::Topology;
use kit as u;
use provider::{
    Auth,
    aws::{
        appsync,
        gateway,
        lambda,
        sfn,
    },
};
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

async fn get_graphql_api_arn(auth: &Auth, name: &str) -> Option<String> {
    let client = appsync::make_client(auth).await;
    appsync::find_api_arn(&client, name).await
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
            lambda::list_tags(&client, &lambda_arn).await.unwrap()
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
        TopologyKind::Routed => {
            let client = gateway::make_client(auth).await;
            gateway::find_tags(&client, &name).await
        }
        TopologyKind::Evented => HashMap::new(),
    }
}

pub fn render(s: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    u::stencil(s, table)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub namespace: String,
    pub kind: String,
    pub sandbox: String,
    pub dir: String,
    pub version: String,
    pub prev_version: String,
    pub git_version: String,
    pub tc_version: String,
    pub updated_at: String,
    pub updated_by: String,
    pub changed: bool,
    pub changelog: Vec<String>
}

fn find_changelog(namespace: &str, dir: &str, from_version: &str, to_version: &str) -> Vec<String> {
    let from_tag = format!("{}-{}", namespace, from_version);
    let to_tag = format!("{}-{}", namespace, to_version);
    let commits_str = tagger::git::changelogs_in_dir(&from_tag, &to_tag, &dir);
    let mut commits: Vec<String> = u::split_lines(&commits_str)
        .iter()
        .map(|s| s.to_string())
        .collect();
    commits.retain(|s| !s.is_empty());
    commits
}

impl Manifest {
    pub async fn new(topology: &Topology, from_auth: &Auth, to_auth: &Auth, sandbox: &str, gen_changelog: bool) -> Manifest {

        let Topology { namespace, kind, fqn, .. } = topology;

        let maybe_rdir = topology.dir.strip_prefix(&format!("{}/", u::root()));
        let dir = match maybe_rdir {
            Some(d) => d,
            None => &topology.dir,
        };

        if gen_changelog {

            let name = render(&fqn, sandbox);
            let from_tags = lookup_tags(from_auth, &kind, &name).await;
            let to_tags = lookup_tags(to_auth, &kind, &name).await;
            let from_version = u::safe_unwrap(from_tags.get("version"));
            let to_version = u::safe_unwrap(to_tags.get("version"));

            let updated_at = u::safe_unwrap(from_tags.get("updated_at"));
            let updated_by = u::safe_unwrap(from_tags.get("updated_by"));

            let tc_version = u::safe_unwrap(from_tags.get("tc_version"));

            let changelog = find_changelog(&namespace, ".", &from_version, &to_version);

            let changed = from_version != to_version;

            Manifest {
                namespace: namespace.to_string(),
                kind: topology.kind.to_str(),
                dir: dir.to_string(),
                sandbox: String::from(sandbox),
                version: from_version,
                prev_version: to_version,
                git_version: String::from(""),
                tc_version: tc_version,
                updated_at: updated_at,
                updated_by: updated_by,
                changed: changed,
                changelog: changelog
            }
        } else {
            Manifest {
                namespace: namespace.to_string(),
                kind: topology.kind.to_str(),
                dir: topology.dir.to_string(),
                sandbox: String::from(sandbox),
                version: topology.version.clone(),
                prev_version: String::from(""),
                git_version: String::from(""),
                tc_version: String::from(""),
                updated_at: String::from(""),
                updated_by: String::from(""),
                changed: false,
                changelog: vec![]
            }
        }
    }
}
