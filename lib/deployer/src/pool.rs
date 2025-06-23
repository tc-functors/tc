use crate::aws::{
    cognito,
    lambda,
};
use authorizer::Auth;
use compiler::topology::Pool;
use std::collections::HashMap;

pub async fn delete(_auth: &Auth, _pools: &HashMap<String, Pool>) {
    // let client = cognito::make_client(auth).await;
    // for pool in pools {
    //     //cognito::delete_pool(&client, &pool).await
    // }
}

fn as_source_arn(auth: &Auth, from: &str) -> String {
    format!(
        "arn:aws:ses:{}:{}:identity/{}",
        auth.region, auth.account, from
    )
}

async fn add_permission(auth: &Auth, lambda_arn: &str, pool_id: &str) {
    let client = lambda::make_client(auth).await;
    let source_arn = auth.pool_arn(pool_id);
    let principal = "cognito-idp.amazonaws.com";
    let _ = lambda::add_permission(client, lambda_arn, principal, &source_arn, pool_id).await;
}

async fn update_functions(auth: &Auth, pool_id: &str, triggers: HashMap<String, String>) {
    for (_, arn) in triggers {
        println!("Updating permission {} ", &arn);
        add_permission(auth, &arn, pool_id).await;
    }
}

pub async fn create(auth: &Auth, pools: &HashMap<String, Pool>) {
    let client = cognito::make_client(auth).await;
    for (name, pool) in pools {
        let mappings = cognito::make_lambda_mappings(pool.triggers.clone());
        let source_arn = as_source_arn(auth, &pool.from_email);
        let email_config = cognito::make_email_config(&pool.from_email, &source_arn);
        let pool_id =
            cognito::create_or_update_pool(&client, &name, mappings.clone(), email_config).await;
        update_functions(auth, &pool_id, pool.triggers.clone()).await
    }
}

pub async fn update(_auth: &Auth, _pools: &HashMap<String, Pool>, _c: &str) {

}
