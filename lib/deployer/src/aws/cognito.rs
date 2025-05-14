use aws_sdk_cognitoidentityprovider::{
    Client,
};
use aws_sdk_cognitoidentityprovider::types::LambdaConfigType;
use aws_sdk_cognitoidentityprovider::types::builders::LambdaConfigTypeBuilder;
use kit::*;
use authorizer::Auth;
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}


pub fn make_lambda_mappings(h: HashMap<String, String>) -> LambdaConfigType {
    let it = LambdaConfigTypeBuilder::default();
    it
        .set_pre_sign_up(h.get("PRE_SIGN_UP").cloned())
        .set_post_confirmation(h.get("POST_CONFIRMATION").cloned())
        .set_pre_authentication(h.get("PRE_AUTHENTICATION").cloned())
        .set_post_authentication(h.get("POST_AUTHENTICATION").cloned())
        .set_create_auth_challenge(h.get("CREATE_AUTH_CHALLENGE").cloned())
        .set_verify_auth_challenge_response(h.get("VERIFY_AUTH_CHALLENGE_RESPONSE").cloned())
        .build()
}

async fn list_pools(client: &Client) -> HashMap<String, String> {
    let res = client
        .list_user_pools()
        .send()
        .await
        .unwrap();
    let xs = res.user_pools.unwrap();
    let mut h: HashMap<String, String> = HashMap::new();
    for x in xs {
        h.insert(x.name.unwrap(), x.id.unwrap());
    }
    h
}

async fn find_pool(client: &Client, name: &str) -> Option<String> {
    let pools = list_pools(client).await;
    pools.get(name).cloned()
}

async fn update_pool(client: &Client, id: &str, triggers: LambdaConfigType) {
    let _ = client
        .update_user_pool()
        .user_pool_id(s!(id))
        .lambda_config(triggers)
        .send()
        .await;
}

pub async fn create_pool(client: &Client, name: &str, triggers: LambdaConfigType) {
    let _ = client
        .create_user_pool()
        .pool_name(s!(name))
        .lambda_config(triggers)
        .send()
        .await;
}

pub async fn create_or_update_pool(client: &Client, name: &str, triggers: LambdaConfigType) {
    let maybe_pool_id = find_pool(client, name).await;
    match maybe_pool_id {
        Some(id) => update_pool(client, &id, triggers).await,
        None => create_pool(client, name, triggers).await
    }
}

pub async fn delete_pool(client: &Client, name: &str) {
    let maybe_pool_id = find_pool(client, name).await;
    match maybe_pool_id {
        Some(id) => {
            let _ = client
                .delete_user_pool()
                .user_pool_id(s!(id))
                .send()
                .await;
        },
        None => println!("{} not found", name)
    }
}
