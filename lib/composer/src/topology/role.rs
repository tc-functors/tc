use super::template;
use crate::Entity;
use kit::*;
use kit as u;
mod trust;
mod policy;
use serde_derive::{
    Deserialize,
    Serialize,
};


use trust::Trust;
use policy::Policy;

fn read_policy(path: &str) -> Policy {
    let data = u::slurp(path);
    let policy: Policy = serde_json::from_str(&data).unwrap();
    policy
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Kind {
    Base,
    Override,
    Provided
}

impl Kind {

    pub fn to_str(&self) -> String {
        match self {
            Kind::Base => s!("base"),
            Kind::Override => s!("override"),
            Kind::Provided => s!("provided")
        }
    }

}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub kind: Kind,
    pub trust: Trust,
    pub arn: String,
    pub policy_name: String,
    pub policy: Policy,
    pub policy_arn: String,
}

impl Role {
    pub fn new(entity: Entity, role_file: &str, fqn: &str) -> Role {
        if u::file_exists(&role_file) {

            let abbr = if fqn.chars().count() > 15 {
                u::abbreviate(fqn, "-")
            } else {
                fqn.to_string()
            };
            let name = format!("tc-{}", abbr);
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
        let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
        let infra_dir = format!("{}/infrastructure/tc/base", &u::root());
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

    pub fn entity_role_arn(entity: Entity) -> String {
        let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
        template::role_arn(&name)
    }


}
