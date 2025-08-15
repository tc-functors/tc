use crate::aws::{
    iam,
    iam::Role,
};
use std::collections::HashMap;
use authorizer::Auth;
use composer;

fn should_delete() -> bool {
    match std::env::var("TC_FORCE_DELETE") {
        Ok(_) => true,
        Err(_) => false
    }
}

pub async fn delete(auth: &Auth, roles: &HashMap<String, composer::Role>) {
    let client = iam::make_client(auth).await;
    for (_, role) in roles {
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

async fn create_aux(profile: String, role_arn: Option<String>, role: composer::Role, tags: HashMap<String, String>) {
    let auth = Auth::new(Some(profile), role_arn).await;
    let client = iam::make_client(&auth).await;
    if &role.kind.to_str() != "provided" {
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

pub async fn create_or_update(auth: &Auth, roles: &HashMap<String, composer::Role>, tags: &HashMap<String, String>) {

    let mut tasks = vec![];

    //println!("Creating roles ({})", roles.len());
    for (_, role) in roles.clone() {
        let tags = tags.clone();
        let p = auth.name.to_string();
        let role_arn = auth.assume_role.to_owned();
        let h = tokio::spawn(async move {
            create_aux(p, role_arn, role.clone(), tags.clone()).await;
        });
        tasks.push(h);
    }
    for task in tasks {
        let _ = task.await;
    }
}
