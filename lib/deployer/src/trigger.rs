use authorizer::Auth;
use crate::aws::cognito;
use std::collections::HashMap;

pub async fn delete(_auth: &Auth, _pools: Vec<String>) {
    // let client = cognito::make_client(auth).await;
    // for pool in pools {
    //     //cognito::delete_pool(&client, &pool).await
    // }
}

pub async fn create(auth: &Auth, pools: Vec<String>, triggers: HashMap<String, String>) {
    let client = cognito::make_client(auth).await;
    let mappings = cognito::make_lambda_mappings(triggers);
    for pool in pools {
        cognito::create_or_update_pool(&client, &pool, mappings.clone()).await
    }
}
