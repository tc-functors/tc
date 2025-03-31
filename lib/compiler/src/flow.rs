use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use super::spec::TopologySpec;
use super::{version, template, role};
use super::{Role, RoleKind};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flow {
    pub name: String,
    pub arn: String,
    pub tags: HashMap<String, String>,
    pub definition: Value,
    pub mode: String,
    pub role: Role
}

fn read_definition(dir: &str, def: Value) -> Value {
    match def.as_str() {
        Some(p) => {
            let path = format!("{}/{}", dir, &p);
            if (path.ends_with(".json") || path.ends_with(".yml")) && u::file_exists(&path) {
                let data = u::slurp(&path);
                u::json_value(&data)
            } else {
                def
            }
        }
        None => def,
    }
}

fn parent_tags_file(dir: &str) -> Option<String> {
    let paths = vec![
        u::absolutize(dir, "../tags.json"),
        u::absolutize(dir, "../../tags.json"),
        u::absolutize(dir, "../../../tags.json"),
        u::absolutize(dir, "../../../../tags.json"),
        s!("../tags.json"),
        s!("../../tags.json"),
        s!("../../../tags.json"),
        s!("../../../../tags.json"),
    ];
    u::any_path(paths)
}

fn load_tags(infra_dir: &str) -> HashMap<String, String> {
    let tags_file = format!("{}/tags.json", infra_dir);
    let parent_file = parent_tags_file(infra_dir);
    if u::file_exists(&tags_file) {
        let data: String = u::slurp(&tags_file);
        let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
        tags
    } else {
        match parent_file {
            Some(f) => {
                let data: String = u::slurp(&f);
                let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
                tags
            }
            None => {
                HashMap::new()
            }
        }
    }
}

fn make_tags(namespace: &str, infra_dir: &str) -> HashMap<String, String> {
    let tc_version = option_env!("PROJECT_VERSION")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();

    let version = version::current_semver(namespace);
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(s!("namespace"), s!(namespace));
    h.insert(s!("sandbox"), format!("{{{{sandbox}}}}"));
    h.insert(s!("version"), version);
    h.insert(s!("git_branch"), version::branch_name());
    h.insert(s!("deployer"), s!("tc"));
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("tc_version"), tc_version);

    let given_tags = load_tags(infra_dir);
    h.extend(given_tags);
    h
}

fn make_role(infra_dir: &str, fqn: &str) -> Role {
    let role_file = format!("{}/roles/sfn.json", infra_dir);
    let role_name = format!("tc-{}-sfn-role", fqn);
    let policy_name = format!("tc-{}-sfn-policy", fqn);
    if u::file_exists(&role_file) {
        Role::new(RoleKind::StepFunction, &role_file, &role_name, &policy_name)
    } else {
        role::default(RoleKind::StepFunction)
    }
}

impl Flow {

    pub fn new(dir: &str, infra_dir: &str, fqn: &str, spec: &TopologySpec) -> Option<Flow> {


        let def = match &spec.flow {
            Some(f) => Some(read_definition(dir, f.clone())),
            None => spec.states.to_owned(),
        };

        let mode = match &spec.mode {
            Some(m) => m.to_string(),
            None => s!("Express")
        };

        let tags = make_tags(&spec.name, infra_dir);

        match def {
            Some(definition) => Some(
                Flow {
                    name: s!(fqn),
                    arn: template::sfn_arn(fqn),
                    tags: tags,
                    definition: definition,
                    mode: mode,
                    role: make_role(infra_dir, fqn)
                }),
            None => None
        }
    }
}
