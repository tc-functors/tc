use aws_sdk_cognitoidentityprovider::{
    Client,
};
use aws_sdk_cognitoidentityprovider::types::LambdaConfigType;
use aws_sdk_cognitoidentityprovider::types::builders::LambdaConfigTypeBuilder;
use aws_sdk_cognitoidentityprovider::types::VerifiedAttributeType;
use aws_sdk_cognitoidentityprovider::types::EmailConfigurationType;
use aws_sdk_cognitoidentityprovider::types::EmailSendingAccountType;
use aws_sdk_cognitoidentityprovider::types::builders::EmailConfigurationTypeBuilder;
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
        .set_custom_message(h.get("CUSTOM_MESSAGE").cloned())
        .build()
}

pub fn make_email_config(from: &str, source_arn: &str) -> EmailConfigurationType {
    let it = EmailConfigurationTypeBuilder::default();
    it
        .email_sending_account(EmailSendingAccountType::Developer)
        .from(String::from(from))
        .source_arn(String::from(source_arn))
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

async fn update_pool(client: &Client, id: &str, triggers: LambdaConfigType, email_config: EmailConfigurationType) -> String {
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
        Err(e) => panic!("{:?}", e)
    }
}

pub async fn create_pool(client: &Client, name: &str, triggers: LambdaConfigType, email_config: EmailConfigurationType) -> String {
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

pub async fn create_or_update_pool(client: &Client, name: &str, triggers: LambdaConfigType, email_config: EmailConfigurationType) -> String {
    let maybe_pool_id = find_pool(client, name).await;
    match maybe_pool_id {
        Some(id) => update_pool(client, &id, triggers, email_config).await,
        None => create_pool(client, name, triggers, email_config).await
    }
}

pub async fn _delete_pool(client: &Client, name: &str) {
    let maybe_pool_id = find_pool(client, name).await;
    match maybe_pool_id {
        Some(id) => {
            println!("Deleting pool {} ({})", name, &id);
            let _ = client
                .delete_user_pool()
                .user_pool_id(s!(id))
                .send()
                .await;
        },
        None => println!("{} not found", name)
    }
}
