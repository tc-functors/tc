use crate::aws::{
    appsync,
    lambda,
};
use authorizer::Auth;
use composer::Mutation;
use std::collections::HashMap;

async fn add_permission(auth: &Auth, statement_id: &str, authorizer_arn: &str) {
    let client = lambda::make_client(auth).await;
    let principal = "appsync.amazonaws.com";
    let _ = lambda::add_permission_basic(client, authorizer_arn, principal, statement_id).await;
}

async fn create_mutation(auth: &Auth, mutation: Mutation, tags: HashMap<String, String>) {
    let Mutation {
        api_name,
        authorizer,
        types,
        resolvers,
        role_arn,
        ..
    } = mutation;
    let authorizer_arn = auth.lambda_arn(&authorizer);
    let client = appsync::make_client(auth).await;
    let (api_id, _) =
        appsync::create_or_update_api(&client, &api_name, &authorizer_arn, tags.clone()).await;

    add_permission(auth, &api_name, &authorizer_arn).await;
    appsync::create_types(auth, &api_id, types).await;

    let client = appsync::make_client(auth).await;
    for (field_name, resolver) in resolvers {
        println!("Creating mutation {}", &field_name);
        let datasource_name = &field_name;
        let kind = &resolver.entity;

        let datasource_input = appsync::DatasourceInput {
            kind: kind.to_str(),
            name: String::from(datasource_name),
            role_arn: role_arn.clone(),
            target_arn: resolver.target_arn.to_owned(),
        };

        appsync::find_or_create_datasource(&client, &api_id, datasource_input).await;
        let _ = appsync::create_or_update_function(&client, &api_id, &field_name, datasource_name)
            .await;
        appsync::find_or_create_resolver(&client, &api_id, &field_name, datasource_name).await;
    }
    appsync::update_tags(&client, &auth.graphql_arn(&api_id), tags.clone()).await;
}

pub async fn create(
    auth: &Auth,
    mutations: &HashMap<String, Mutation>,
    tags: &HashMap<String, String>,
) {
    for (_, mutation) in mutations {
        create_mutation(auth, mutation.clone(), tags.clone()).await;
    }
}

pub async fn delete(auth: &Auth, mutations: &HashMap<String, Mutation>) {
    for (_, mutation) in mutations {
        let Mutation { api_name, .. } = mutation;
        let client = appsync::make_client(auth).await;
        appsync::delete_api(&client, &api_name).await;
    }
}

pub async fn update(_auth: &Auth, _mutations: &HashMap<String, Mutation>, _c: &str) {
    todo!()
}
