use compiler;
use aws::iam;
use aws::iam::Role;
use aws::Env;

pub async fn delete(env: &Env, roles: &Vec<compiler::Role>) {
    let client = iam::make_client(env).await;
    for role in roles {
        let r = Role {
            client: client.clone(),
            name: role.name.clone(),
            trust_policy: role.trust.to_string(),
            policy_arn: role.policy_arn.clone(),
            policy_name: role.policy_name.clone(),
            policy_doc: role.policy.to_string(),
        };
        let _ = r.delete().await;
    }
}

pub async fn create_or_update(env: &Env, roles: &Vec<compiler::Role>) {
    let client = iam::make_client(env).await;
    for role in roles {
        let r = Role {
            client: client.clone(),
            name: role.name.clone(),
            trust_policy: role.trust.to_string(),
            policy_arn: role.policy_arn.clone(),
            policy_name: role.policy_name.clone(),
            policy_doc: role.policy.to_string(),
        };
        let _ = r.create_or_update().await;
    }
}
