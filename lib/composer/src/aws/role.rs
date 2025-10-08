use super::template;
use crate::{aws::role, Entity};
use kit as u;
use kit::*;
pub mod policy;
mod trust;
use policy::Policy;
use serde_derive::{
    Deserialize,
    Serialize,
};
use crate::aws::role::policy::Action;
use trust::Trust;
use tracing::debug;
use std::{
    fs,
    io::{
        ErrorKind
    }
};

static IAM_POLICY_VERSION: &str = "2012-10-17";

impl Default for Policy { 
    fn default() -> Self {
        Policy { 
            version: IAM_POLICY_VERSION.to_string(),
            statement: vec![],
        }
    }
}

fn read_policy(path: &str) -> Policy {
    tracing::debug!("Reading {}", path);
    let data = u::slurp(path);
    let policy: Policy = serde_json::from_str(&data).unwrap();
    policy
}

pub fn read_policy_file(path: &str) -> String {
    match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => Policy::default().to_string(), 
        Err(error) => {
            debug!("Error reading file: {}", error);
            Policy::default().to_string()
        }
    }
}

fn get_policy_statements(path: &str) -> (Vec<Action>, String) {
    tracing::debug!("Reading {}", path);
    let topology_policy = read_policy_file(path);
    let policy: Policy = serde_json::from_str(&topology_policy).unwrap();

    (policy.statement, policy.version)
}

fn generate_topology_policy(infra_dir: &str, entity: Entity, role_file: &str) -> Policy {
    let base_roles_dir = format!("{}/{}/base/roles", &u::root(), infra_dir);
    let base_role_json_file = format!("{}/{}.json", base_roles_dir, &entity.to_str());

    let (mut base_statements, version) = get_policy_statements(&base_role_json_file);
    let (topology_statements, _) = get_policy_statements(&role_file);

    base_statements.extend(topology_statements);

    let policy = Policy {
        version: String::from(version),
        statement: Vec::<Action>::from(base_statements)
    };

    policy 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Kind {
    Base,
    Override,
    Provided,
}

impl Kind {
    pub fn to_str(&self) -> String {
        match self {
            Kind::Base => s!("base"),
            Kind::Override => s!("override"),
            Kind::Provided => s!("provided"),
        }
    }
}

fn find_legacy_role_name(entity: Entity) -> String {
    match entity {
        Entity::Function => s!("tc-base-lambda-role"),
        Entity::Event => s!("tc-base-event-role"),
        Entity::Route => s!("tc-base-api-role"),
        Entity::Mutation => s!("tc-base-appsync-role"),
        Entity::State => s!("tc-base-sfn-role"),
        _ => s!("tc-base-lambda-role"),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Role {
    pub name: String,
    pub kind: Kind,
    pub trust: Trust,
    pub arn: String,
    pub policy_name: String,
    pub policy: Policy,
    pub policy_arn: String,
}

fn legacy_name_of(entity: Entity) -> String {
    match entity {
        Entity::Route => s!("tc-base-api-role"),
        Entity::Event => s!("tc-base-event-role"),
        Entity::Mutation => s!("tc-base-appsync-role"),
        Entity::State => s!("tc-base-sfn-role"),
        _ => s!("tc-base-lambda-role"),
    }
}

impl Role {
    pub fn new(entity: Entity, role_file: &str, namespace: &str, entity_name: &str) -> Role {
        if u::file_exists(&role_file) {
            let abbr = if entity_name.chars().count() > 10 {
                u::abbreviate(entity_name, "-")
            } else {
                entity_name.to_string()
            };
            let name = format!("tc-{}-{}-{{{{sandbox}}}}", namespace, abbr);
            Role {
                name: s!(&name),
                kind: Kind::Override,
                trust: Trust::new(),
                arn: template::role_arn(&name),
                policy: generate_topology_policy("infrastructure/tc", entity, &role_file),
                policy_name: s!(&name),
                policy_arn: template::policy_arn(&name),
            }
        } else {
            let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
            Role {
                name: s!(&name),
                kind: Kind::Base,
                trust: Trust::new(),
                arn: template::role_arn(&name),
                policy: Policy::new(entity),
                policy_name: s!(name),
                policy_arn: template::policy_arn(&name),
            }
        }
    }

    pub fn new_static(
        entity: Entity,
        role_file: &str,
        _namespace: &str,
        entity_name: &str,
    ) -> Role {
        if u::file_exists(&role_file) {
            let name = entity_name;
            Role {
                name: s!(&name),
                kind: Kind::Override,
                trust: Trust::new(),
                arn: template::role_arn(&name),
                policy: read_policy(&role_file),
                policy_name: s!(&name),
                policy_arn: template::policy_arn(&name),
            }
        } else {
            let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
            Role {
                name: s!(&name),
                kind: Kind::Base,
                trust: Trust::new(),
                arn: template::role_arn(&name),
                policy: Policy::new(entity),
                policy_name: s!(name),
                policy_arn: template::policy_arn(&name),
            }
        }
    }

    pub fn default(entity: Entity) -> Role {
        match std::env::var("TC_LEGACY_ROLES") {
            Ok(_) => {
                let name = find_legacy_role_name(entity.clone());
                Role {
                    name: s!(&name),
                    kind: Kind::Provided,
                    trust: Trust::new(),
                    arn: template::role_arn(&name),
                    policy: Policy::new(entity),
                    policy_name: s!(&name),
                    policy_arn: template::policy_arn(&name),
                }
            }

            Err(_) => {
                let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
                let infra_dir = format!("{}/infrastructure/tc/base/roles", &u::root());
                let maybe_base_path = format!("{}/{}.json", infra_dir, &entity.to_str());
                let policy = if u::file_exists(&maybe_base_path) {
                    read_policy(&maybe_base_path)
                } else {
                    Policy::new(entity)
                };
                Role {
                    name: s!(&name),
                    kind: Kind::Base,
                    trust: Trust::new(),
                    arn: template::role_arn(&name),
                    policy: policy,
                    policy_name: s!(&name),
                    policy_arn: template::policy_arn(&name),
                }
            }
        }
    }

    pub fn provided(name: &str) -> Role {
        Role {
            name: s!(name),
            kind: Kind::Provided,
            trust: Trust::new(),
            arn: template::role_arn(&name),
            policy: Policy::new(Entity::Function),
            policy_name: s!(&name),
            policy_arn: template::policy_arn(&name),
        }
    }

    pub fn provided_by_entity(entity: Entity) -> Role {
        let name = legacy_name_of(entity);
        Role {
            name: s!(name),
            kind: Kind::Provided,
            trust: Trust::new(),
            arn: template::role_arn(&name),
            policy: Policy::new(Entity::Function),
            policy_name: s!(&name),
            policy_arn: template::policy_arn(&name),
        }
    }

    pub fn entity_role_arn(entity: Entity) -> String {
        match std::env::var("TC_LEGACY_ROLES") {
            Ok(_) => {
                let name = legacy_name_of(entity);
                template::role_arn(&name)
            }
            Err(_) => {
                let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
                template::role_arn(&name)
            }
        }
    }
}
