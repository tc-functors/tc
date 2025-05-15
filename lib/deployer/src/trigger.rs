use authorizer::Auth;
use crate::aws::cognito;
use std::collections::HashMap;
use kit as u;

fn abbr(name: &str) -> String {
    if name.chars().count() > 15 {
        u::abbreviate(name, "-")
    } else {
        name.to_string()
    }
}

pub async fn delete(auth: &Auth, fqn: &str) {
    let fqn = abbr(fqn);
    let client = cognito::make_client(auth).await;
    println!("Deleting triggers (cognito) {}", &fqn);
    cognito::delete_pool(&client, &fqn).await
}

pub async fn create(auth: &Auth, fqn: &str, triggers: HashMap<String, String>) {
    let client = cognito::make_client(auth).await;
    let fqn = abbr(fqn);
    let mappings = cognito::make_lambda_mappings(triggers);
    println!("Creating triggers (cognito) {}", &fqn);
    cognito::create_or_update_pool(&client, &fqn, mappings).await
}
