use crate::resolver;
use aws::iam;
use aws::iam::Role;
use aws::Env;
use std::collections::HashMap;

pub async fn create(env: &Env, roles: HashMap<String, resolver::Role>) {
    let client = iam::make_client(env).await;
    for (_, role) in roles {
        let r = Role {
            client: client.clone(),
            name: role.name,
            trust_policy: role.trust.to_string(),
            policy_arn: role.policy_arn,
            policy_name: role.policy_name,
            policy_doc: role.policy.to_string(),
        };
        let _ = r.create().await;
    }
}

pub async fn delete(env: &Env, roles: HashMap<String, resolver::Role>) {
    let client = iam::make_client(env).await;
    for (_, role) in roles {
        let r = Role {
            client: client.clone(),
            name: role.name,
            trust_policy: role.trust.to_string(),
            policy_arn: role.policy_arn,
            policy_name: role.policy_name,
            policy_doc: role.policy.to_string(),
        };
        let _ = r.delete().await;
    }
}

pub async fn update(env: &Env, roles: HashMap<String, resolver::Role>) {
    let client = iam::make_client(env).await;
    for (_, role) in roles {
        let r = Role {
            client: client.clone(),
            name: role.name,
            trust_policy: role.trust.to_string(),
            policy_arn: role.policy_arn,
            policy_name: role.policy_name,
            policy_doc: role.policy.to_string(),
        };
        let _ = r.update().await;
    }
}
