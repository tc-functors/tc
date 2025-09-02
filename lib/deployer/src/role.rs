use crate::aws::{
    iam,
    iam::Role,
};
use authorizer::Auth;
use composer;
use std::collections::HashMap;

fn should_delete() -> bool {
    match std::env::var("TC_FORCE_DELETE") {
        Ok(_) => true,
        Err(_) => false,
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
                tags: None,
            };
            let _ = r.delete().await;
        }
    }
}

async fn create_aux(
    profile: String,
    role_arn: Option<String>,
    role: composer::Role,
    tags: HashMap<String, String>,
) {
    let auth = Auth::new(Some(profile), role_arn).await;
    let client = iam::make_client(&auth).await;

    let r = Role {
        client: client.clone(),
        name: role.name.clone(),
        trust_policy: role.trust.to_string(),
        policy_arn: role.policy_arn.clone(),
        policy_name: role.policy_name.clone(),
        policy_doc: role.policy.to_string(),
        tags: Some(iam::make_tags(tags.clone())),
    };

    let _ = match role.kind.to_str().as_ref() {
        "provided" => (),
        "base" => match std::env::var("TC_UPDATE_BASE_ROLES") {
            Ok(_) => {
                let _ = r.create_or_update().await;
                ()
            }
            Err(_) => {
                r.find_or_create().await;
                ()
            }
        },
        _ => {
            let _ = r.create_or_update().await;
            ()
        }
    };
}

pub async fn create_or_update(
    auth: &Auth,
    roles: &HashMap<String, composer::Role>,
    tags: &HashMap<String, String>,
) {
    let mut tasks = vec![];

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
