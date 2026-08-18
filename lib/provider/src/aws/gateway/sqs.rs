use aws_sdk_apigatewayv2::{
    Client,
    Error,
    types::{
        ConnectionType,
        IntegrationType,
    },
};
use kit::*;
use std::collections::HashMap;

// Keyed by description (the route key): SQS-SendMessage takes no Name parameter,
// and QueueUrl alone collapses routes that share a queue. Integrations predating
// the description are matched by QueueUrl so they are updated, not orphaned.
async fn find(client: &Client, api_id: &str, key: &str, queue_url: &str) -> Option<String> {
    let r = client
        .get_integrations()
        .api_id(api_id.to_string())
        .max_results(s!("1000"))
        .send()
        .await
        .unwrap();
    let ints = match r.items {
        Some(ints) => ints,
        None => return None,
    };
    for int in &ints {
        if int.description == Some(s!(key)) {
            return int.integration_id.clone();
        }
    }
    for int in &ints {
        let url = match &int.request_parameters {
            Some(req) => req.get("QueueUrl").cloned(),
            None => None,
        };
        if int.description.is_none() && url == Some(s!(queue_url)) {
            return int.integration_id.clone();
        }
    }
    None
}

async fn create(
    client: &Client,
    api_id: &str,
    key: &str,
    role_arn: &str,
    request_parameters: HashMap<String, String>,
) -> Result<String, Error> {
    let res = client
        .create_integration()
        .api_id(s!(api_id))
        .description(s!(key))
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("1.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_subtype(s!("SQS-SendMessage"))
        .set_request_parameters(Some(request_parameters))
        .send()
        .await;
    match res {
        Ok(r) => Ok(r.integration_id.unwrap()),
        Err(e) => panic!("{:?}", e),
    }
}

async fn update(
    client: &Client,
    id: &str,
    api_id: &str,
    key: &str,
    role_arn: &str,
    request_parameters: HashMap<String, String>,
) -> Result<String, Error> {
    let res = client
        .update_integration()
        .api_id(s!(api_id))
        .integration_id(s!(id))
        .description(s!(key))
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("1.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_subtype(s!("SQS-SendMessage"))
        .set_request_parameters(Some(request_parameters))
        .send()
        .await;
    match res {
        Ok(r) => Ok(r.integration_id.unwrap()),
        Err(e) => panic!("{:?}", e),
    }
}

// Update on a hit, like the lambda integration does - a route found by key may
// still carry stale request-parameters from an earlier deploy.
pub async fn create_or_update(
    client: &Client,
    api_id: &str,
    key: &str,
    role_arn: &str,
    request_parameters: HashMap<String, String>,
    queue_url: &str,
) -> String {
    let maybe_int = find(client, api_id, key, queue_url).await;
    match maybe_int {
        Some(id) => {
            tracing::debug!("Found SQS Integration {}", id);
            let _ = update(client, &id, api_id, key, role_arn, request_parameters).await;
            id
        }
        _ => {
            let id = create(client, api_id, key, role_arn, request_parameters)
                .await
                .unwrap();
            tracing::debug!("Created SQS Integration {}", id);
            id
        }
    }
}

pub async fn delete(client: &Client, api_id: &str, key: &str, queue_url: &str) {
    let maybe_int = find(client, api_id, key, queue_url).await;
    match maybe_int {
        Some(id) => {
            let _ = client
                .delete_integration()
                .api_id(s!(api_id))
                .integration_id(id)
                .send()
                .await;
        }
        _ => (),
    }
}
