use crate::Auth;
use aws_sdk_cognitoidentityprovider::{
    Client,
    types::{
        EmailConfigurationType,
        EmailSendingAccountType,
        LambdaConfigType,
        VerifiedAttributeType,
        builders::{
            EmailConfigurationTypeBuilder,
            LambdaConfigTypeBuilder,
        },
    },
};
use kit::*;
use std::collections::HashMap;

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub fn make_lambda_mappings(h: HashMap<String, String>) -> LambdaConfigType {
    let it = LambdaConfigTypeBuilder::default();
    it.set_pre_sign_up(h.get("PRE_SIGN_UP").cloned())
        .set_post_confirmation(h.get("POST_CONFIRMATION").cloned())
        .set_pre_authentication(h.get("PRE_AUTHENTICATION").cloned())
        .set_post_authentication(h.get("POST_AUTHENTICATION").cloned())
        .set_create_auth_challenge(h.get("CREATE_AUTH_CHALLENGE").cloned())
        .set_verify_auth_challenge_response(h.get("VERIFY_AUTH_CHALLENGE_RESPONSE").cloned())
        .set_custom_message(h.get("CUSTOM_MESSAGE").cloned())
        .build()
}

pub fn make_email_config(from: &str, source_arn: &str) -> EmailConfigurationType {
    let it = EmailConfigurationTypeBuilder::default();
    it.email_sending_account(EmailSendingAccountType::Developer)
        .from(String::from(from))
        .source_arn(String::from(source_arn))
        .build()
}

async fn list_pools(client: &Client) -> HashMap<String, String> {
    let res = client.list_user_pools().send().await.unwrap();
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

async fn update_pool(
    client: &Client,
    id: &str,
    triggers: LambdaConfigType,
    email_config: EmailConfigurationType,
) -> String {
    println!("Updating pool ({})", id);
    let res = client
        .update_user_pool()
        .user_pool_id(s!(id))
        .lambda_config(triggers)
        .auto_verified_attributes(VerifiedAttributeType::Email)
        .email_configuration(email_config)
        .send()
        .await;
    match res {
        Ok(_) => id.to_string(),
        Err(e) => panic!("{:?}", e),
    }
}

pub async fn create_pool(
    client: &Client,
    name: &str,
    triggers: LambdaConfigType,
    email_config: EmailConfigurationType,
) -> String {
    println!("Creating pool {}", name);
    let res = client
        .create_user_pool()
        .pool_name(s!(name))
        .lambda_config(triggers)
        .auto_verified_attributes(VerifiedAttributeType::Email)
        .email_configuration(email_config)
        .send()
        .await;
    res.unwrap().user_pool.unwrap().id.expect("Not found")
}

pub async fn create_or_update_pool(
    client: &Client,
    name: &str,
    triggers: LambdaConfigType,
    email_config: EmailConfigurationType,
) -> String {
    let maybe_pool_id = find_pool(client, name).await;
    match maybe_pool_id {
        Some(id) => update_pool(client, &id, triggers, email_config).await,
        None => create_pool(client, name, triggers, email_config).await,
    }
}

pub async fn _delete_pool(client: &Client, name: &str) {
    let maybe_pool_id = find_pool(client, name).await;
    match maybe_pool_id {
        Some(id) => {
            println!("Deleting pool {} ({})", name, &id);
            let _ = client.delete_user_pool().user_pool_id(s!(id)).send().await;
        }
        None => println!("{} not found", name),
    }
}

// generic jwt pool

async fn list_app_clients(client: &Client, pool_id: &str) -> HashMap<String, String> {
    let res = client
        .list_user_pool_clients()
        .user_pool_id(pool_id)
        .send()
        .await
        .unwrap();
    let clients = res.user_pool_clients.unwrap();
    let mut h: HashMap<String, String> = HashMap::new();
    for c in clients {
        h.insert(c.client_name.unwrap(), c.client_id.unwrap());
    }
    h
}

async fn find_app_client(client: &Client, pool_id: &str, client_name: &str) -> Option<String> {
    let clients = list_app_clients(client, pool_id).await;
    clients.get(client_name).cloned()
}

async fn create_app_client(client: &Client, pool_id: &str, client_name: &str) -> String {
    let res = client
        .create_user_pool_client()
        .user_pool_id(pool_id)
        .client_name(client_name)
        .send()
        .await
        .unwrap();

    res.user_pool_client.unwrap().client_id.unwrap()
}

pub async fn find_or_create_app_client(
    client: &Client,
    pool_id: &str,
    client_name: &str,
) -> String {
    let maybe_client_app_id = find_app_client(client, pool_id, client_name).await;
    match maybe_client_app_id {
        Some(id) => id,
        None => create_app_client(client, pool_id, client_name).await,
    }
}

async fn update_auth_pool(client: &Client, id: &str) -> String {
    println!("Updating pool ({})", id);
    let res = client
        .update_user_pool()
        .user_pool_id(s!(id))
        .auto_verified_attributes(VerifiedAttributeType::Email)
        .send()
        .await;
    match res {
        Ok(_) => id.to_string(),
        Err(e) => panic!("{:?}", e),
    }
}

async fn create_auth_pool(client: &Client, name: &str) -> String {
    println!("Creating pool {}", name);
    let res = client
        .create_user_pool()
        .pool_name(s!(name))
        .auto_verified_attributes(VerifiedAttributeType::Email)
        .send()
        .await;
    res.unwrap().user_pool.unwrap().id.expect("Not found")
}

pub async fn create_or_update_auth_pool(client: &Client, name: &str) -> (String, String) {
    let maybe_pool_id = find_pool(client, name).await;
    let id = match maybe_pool_id {
        Some(id) => update_auth_pool(client, &id).await,
        None => create_auth_pool(client, name).await,
    };
    let client_name = format!("client_{}", &name);
    let client_id = find_or_create_app_client(client, &id, &client_name).await;
    (id, client_id)
}

pub async fn get_config(client: &Client, pool_name: &str) -> (Option<String>, Option<String>) {
    let maybe_pool_id = find_pool(client, pool_name).await;
    match maybe_pool_id {
        Some(pool_id) => {
            let client_name = format!("client_{}", &pool_name);
            let client_id = find_app_client(client, &pool_id, &client_name).await;
            (Some(pool_id), client_id)
        }
        None => (None, None),
    }
}
