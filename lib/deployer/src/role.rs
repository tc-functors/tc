use crate::aws::{
    iam,
    iam::Role,
};
use std::collections::HashMap;
use authorizer::Auth;
use composer;

fn should_delete() -> bool {
    match std::env::var("TC_PRUNE_ROLES") {
        Ok(_) => true,
        Err(_) => false
    }
}

pub async fn delete(auth: &Auth, roles: &Vec<composer::Role>) {
    let client = iam::make_client(auth).await;
    for role in roles {
        if &role.kind.to_str() == "override" || should_delete() {
            let r = Role {
                client: client.clone(),
                name: role.name.clone(),
                trust_policy: role.trust.to_string(),
                policy_arn: role.policy_arn.clone(),
                policy_name: role.policy_name.clone(),
                policy_doc: role.policy.to_string(),
                tags: None
            };
            let _ = r.delete().await;
        }
    }
}

pub async fn create_or_update(auth: &Auth, roles: &Vec<composer::Role>, tags: &HashMap<String, String>) {
    let client = iam::make_client(auth).await;
    for role in roles {
        let r = Role {
            client: client.clone(),
            name: role.name.clone(),
            trust_policy: role.trust.to_string(),
            policy_arn: role.policy_arn.clone(),
            policy_name: role.policy_name.clone(),
            policy_doc: role.policy.to_string(),
            tags: Some(iam::make_tags(tags.clone()))
        };
        let _ = r.create_or_update().await;
    }
}
