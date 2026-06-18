use composer::Mutation;
use compiler::Entity;
use kit::*;
use kit as u;
use provider::{
    Auth,
    aws::{
        appsync,
        appsync::AppsyncClient,
        lambda,
        lambda::LambdaClient
    },
};
use std::collections::HashMap;

async fn add_permission(auth: &Auth, statement_id: &str, authorizer_arn: &str) {
    let client = lambda::make_client(auth).await;
    let principal = "appsync.amazonaws.com";
    let _ = lambda::add_permission_basic(client, authorizer_arn, principal, statement_id).await;
}

async fn find_alias_arn(client: &LambdaClient, kind: &Entity, name: &str, arn: &str) -> String {
    match kind {
        Entity::Function => {
            let maybe_alias_arn = lambda::find_alias_arn(&client, name).await;
            match maybe_alias_arn {
                Some(a) => {
                    println!("Using alias function={}", &a);
                    a
                }
                None => arn.to_string()
            }
        },
        _ => arn.to_string()
    }
}

async fn create_mutation(
    auth: &Auth,
    client: &AppsyncClient,
    mutation: Mutation,
    tags: HashMap<String, String>,
) {
    let Mutation {
        api_name,
        authorizer,
        types,
        resolvers,
        role_arn,
        ..
    } = mutation;
    let lc = lambda::make_client(auth).await;
    let authorizer_arn = find_alias_arn(&lc, &Entity::Function, &authorizer, &auth.lambda_arn(&authorizer)).await;
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
            target_arn: find_alias_arn(&lc, kind, &resolver.target_name, &resolver.target_arn).await
        };

        appsync::find_or_create_datasource(&client, &api_id, datasource_input).await;
        let _ = appsync::create_or_update_function(&client, &api_id, &field_name, datasource_name)
            .await;
        appsync::find_or_create_resolver(&client, &api_id, &field_name, datasource_name).await;
    }
    appsync::update_tags(&client, &auth.graphql_api_arn(&api_id), tags.clone()).await;
    let _ = appsync::create_or_update_api_key(&client, &api_id).await;
}

pub async fn create(
    auth: &Auth,
    mutations: &HashMap<String, Mutation>,
    tags: &HashMap<String, String>,
) {
    let client = appsync::make_client(auth).await;
    for (_, mutation) in mutations {
        create_mutation(auth, &client, mutation.clone(), tags.clone()).await;
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

pub async fn list(auth: &Auth, name: &str) {
    let client = appsync::make_client(auth).await;
    let api = appsync::find_graphql_api(&client, name).await;
    match api {
        Some(a) => {
            println!("id: {}", &a.id);
            println!("https: {}", &a.https);
            println!("wss: {}", &a.wss);
        }
        _ => (),
    }
}

pub async fn config(auth: &Auth, name: &str) -> HashMap<String, String> {
    let client = appsync::make_client(auth).await;
    let api = appsync::find_graphql_api(&client, name).await;
    match api {
        Some(a) => {
            let mut h: HashMap<String, String> = HashMap::new();
            let host = str::replace(&a.https, "https://", "").replace("/graphql", "");
            let rhost = str::replace(&a.wss, "wss://", "").replace("/graphql", "");
            h.insert(s!("GRAPHQL_ID"), a.id.clone());
            h.insert(s!("GRAPHQL_ENDPOINT"), a.https.clone());
            h.insert(s!("GRAPHQL_HOST"), host);
            h.insert(s!("GRAPHQL_WSS_ENDPOINT"), a.wss.clone());
            h.insert(s!("GRAPHQL_REALTIME_HOST"), rhost);
            let keys = appsync::list_api_keys(&client, &a.id).await;
            if let Some(key) = keys.first() {
                h.insert(s!("GRAPHQL_API_KEY"), key.to_string());
            }
            h
        }
        _ => HashMap::new(),
    }
}

pub async fn create_dry_run(mutations: &HashMap<String, Mutation>) {
    for (_, mutation) in mutations {
        for (name, _) in &mutation.resolvers {
            println!("Creating mutation: {}", name)
        }
    }
}



pub async fn freeze(auth: &Auth, fqn: &str) {
    let client = appsync::make_client(auth).await;
    let maybe_api = appsync::find_graphql_api(&client, fqn).await;
    if let Some(api) = maybe_api {
        let arn = &auth.graphql_api_arn(&api.id);
        let version = appsync::get_tag(&client, &arn, s!("version")).await;
        if &version != "0.0.1" && !&version.is_empty() {
            println!("Freezing mutation {} ({})", fqn, version);
            let kv = u::kv("freeze", "true");
            let _ = appsync::update_tags(&client, &arn, kv).await;
        }
    }
}

pub async fn unfreeze(auth: &Auth, fqn: &str) {
    let client = appsync::make_client(auth).await;
    let maybe_api = appsync::find_graphql_api(&client, fqn).await;
    if let Some(api) = maybe_api {
        let arn = &auth.graphql_api_arn(&api.id);
        let version = appsync::get_tag(&client, &arn, s!("version")).await;
        if &version != "0.0.1" && !&version.is_empty() {
            println!("Unfreezing mutation {} ({})", fqn, version);
            let kv = u::kv("freeze", "false");
            let _ = appsync::update_tags(&client, &arn, kv).await;
        }
    }
}

pub async fn is_frozen(auth: &Auth, fqn: &str) -> bool {
    let client = appsync::make_client(auth).await;
    let maybe_api = appsync::find_graphql_api(&client, fqn).await;
    if let Some(api) = maybe_api {
        let arn = &auth.graphql_api_arn(&api.id);
        let v = appsync::get_tag(&client, &arn, s!("freeze")).await;
        v == "true"
    } else {
        false
    }
}
