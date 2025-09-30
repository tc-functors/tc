use super::template;
use kit::*;
mod policy;
mod trust;

use serde_derive::{
    Deserialize,
    Serialize,
};

use compiler::Entity;
use compiler::spec::role::{Kind, RoleSpec};

use policy::Policy;
use trust::Trust;
use serde_json::Value;

fn read_policy(v: Value) -> Policy {
    let policy: Policy = serde_json::from_value(v).unwrap();
    policy
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

impl Role {

    pub fn new(spec: &RoleSpec) -> Role {

        let policy = match &spec.policy {
            Some(p) => read_policy(p.clone()),
            None => Policy::new(spec.entity.clone())
        };

        let name = &spec.name;
        Role {
            name: spec.name.clone(),
            kind: spec.kind.clone(),
            trust: Trust::new(),
            arn: template::role_arn(&name),
            policy: policy,
            policy_name: s!(&name),
            policy_arn: template::policy_arn(&name)
        }

    }

    pub fn entity_role_arn(entity: Entity) -> String {
        match std::env::var("TC_LEGACY_ROLES") {
            Ok(_) => {
                let name = compiler::spec::role::find_legacy_role_name(entity);
                template::role_arn(&name)
            }
            Err(_) => {
                let name = format!("tc-base-{}-{{{{sandbox}}}}", &entity.to_str());
                template::role_arn(&name)
            }
        }
    }
}
