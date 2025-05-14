use authorizer::Auth;
use crate::aws::cognito;
use std::collections::HashMap;

pub async fn delete(auth: &Auth, fqn: &str) {
    let client = cognito::make_client(auth).await;
    println!("Deleting triggers (cognito) {}", fqn);
    cognito::delete_pool(&client, fqn).await
}

pub async fn create(auth: &Auth, fqn: &str, triggers: HashMap<String, String>) {
    let client = cognito::make_client(auth).await;
    let mappings = cognito::make_lambda_mappings(triggers);
    println!("Creating triggers (cognito) {}", fqn);
    cognito::create_or_update_pool(&client, fqn, mappings).await
}
