use crate::Entity;
use kit as u;
use kit::*;

use serde_derive::{
    Deserialize,
    Serialize,
};

use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Kind {
    Base,
    Override,
    Provided,
    Legacy
}

impl Kind {
    pub fn to_str(&self) -> String {
        match self {
            Kind::Base => s!("base"),
            Kind::Override => s!("override"),
            Kind::Provided => s!("provided"),
            Kind::Legacy => s!("legacy"),
        }
    }
}

pub fn find_legacy_role_name(entity: Entity) -> String {
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
pub struct RoleSpec {
    pub name: String,
    pub entity: Entity,
    pub kind: Kind,
    pub policy_name: String,
    pub policy: Option<Value>,
}

fn read_policy(path: &str) -> Value {
    tracing::debug!("Reading {}", path);
    let data = u::slurp(path);
    let policy: Value = serde_json::from_str(&data).unwrap();
    policy
}

fn name_of(namespace: &str, s: &str) -> String {
    let abbr = if s.chars().count() > 10 {
        u::abbreviate(s, "-")
    } else {
        s.to_string()
    };
    format!("tc-{}-{}-{{{{sandbox}}}}", namespace, abbr)
}

impl RoleSpec {
    pub fn new(entity: Entity, role_file: &str, namespace: &str, entity_name: &str) -> RoleSpec {
        if u::file_exists(&role_file) {
            let name = name_of(namespace, entity_name);
            RoleSpec {
                name: s!(&name),
                entity: entity,
                kind: Kind::Override,
                policy: Some(read_policy(&role_file)),
                policy_name: s!(&name),
            }
        } else {
            let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
            RoleSpec {
                name: s!(&name),
                kind: Kind::Base,
                entity: entity,
                policy: None,
                policy_name: s!(name),
            }
        }
    }

    pub fn new_static(
        entity: Entity,
        role_file: &str,
        entity_name: &str,
    ) -> RoleSpec {
        if u::file_exists(&role_file) {
            let name = entity_name;
            RoleSpec {
                name: s!(&name),
                kind: Kind::Override,
                entity : entity,
                policy: Some(read_policy(&role_file)),
                policy_name: s!(&name),
            }
        } else {
            let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
            RoleSpec {
                name: s!(&name),
                entity: entity,
                kind: Kind::Base,
                policy: None,
                policy_name: s!(name),
            }
        }
    }

    pub fn default(entity: Entity) -> RoleSpec {
        match std::env::var("TC_LEGACY_ROLES") {
            Ok(_) => {
                let name = find_legacy_role_name(entity.clone());
                RoleSpec {
                    name: s!(&name),
                    entity: entity,
                    kind: Kind::Provided,
                    policy: None,
                    policy_name: s!(&name),
                }
            }

            Err(_) => {
                let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
                let infra_dir = format!("{}/infrastructure/tc/base/roles", &u::root());
                let maybe_base_path = format!("{}/{}.json", infra_dir, &entity.to_str());
                let policy = if u::file_exists(&maybe_base_path) {
                    Some(read_policy(&maybe_base_path))
                } else {
                    None
                };
                RoleSpec {
                    name: s!(&name),
                    entity: entity,
                    kind: Kind::Base,
                    policy: policy,
                    policy_name: s!(&name),
                }
            }
        }
    }

    pub fn provided(name: &str) -> RoleSpec {
        RoleSpec {
            name: s!(name),
            entity: Entity::Function,
            kind: Kind::Provided,
            policy: None,
            policy_name: s!(&name),
        }
    }

    pub fn provided_by_entity(entity: Entity) -> RoleSpec {
        let name = find_legacy_role_name(entity.clone());
        RoleSpec {
            name: s!(name),
            entity: entity,
            kind: Kind::Provided,
            policy: None,
            policy_name: s!(&name),
        }
    }

}
