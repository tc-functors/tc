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

// SQS-SendMessage integrations carry no Name request-parameter (the subtype only
// accepts QueueUrl/MessageBody/DelaySeconds/MessageGroupId/...), so the queue url
// is what identifies one route's integration.
async fn find(client: &Client, api_id: &str, queue_url: &str) -> Option<String> {
    let r = client
        .get_integrations()
        .api_id(api_id.to_string())
        .max_results(s!("1000"))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(ints) => {
            for int in ints.to_vec() {
                let url = match &int.request_parameters {
                    Some(req) => req.get("QueueUrl").cloned(),
                    None => None,
                };
                if url == Some(s!(queue_url)) {
                    return int.integration_id;
                }
            }
            return None;
        }
        None => None,
    }
}

async fn create(
    client: &Client,
    api_id: &str,
    role_arn: &str,
    request_parameters: HashMap<String, String>,
) -> Result<String, Error> {
    let subtype = s!("SQS-SendMessage");

    let res = client
        .create_integration()
        .api_id(s!(api_id))
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("1.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_subtype(subtype)
        .set_request_parameters(Some(request_parameters))
        .send()
        .await;
    match res {
        Ok(r) => Ok(r.integration_id.unwrap()),
        Err(e) => panic!("{:?}", e),
    }
}

pub async fn find_or_create(
    client: &Client,
    api_id: &str,
    role_arn: &str,
    request_parameters: HashMap<String, String>,
    queue_url: &str,
) -> String {
    let maybe_int = find(client, api_id, queue_url).await;
    match maybe_int {
        Some(id) => id,
        _ => create(client, api_id, role_arn, request_parameters)
            .await
            .unwrap(),
    }
}

pub async fn delete(client: &Client, api_id: &str, queue_url: &str) {
    let maybe_int = find(client, api_id, queue_url).await;
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
