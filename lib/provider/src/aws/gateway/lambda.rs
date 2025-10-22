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

async fn find(client: &Client, api_id: &str, lambda_arn: &str) -> Option<String> {
    let r = client
        .get_integrations()
        .api_id(api_id.to_string())
        .max_results(s!("2000"))
        .send()
        .await
        .unwrap();
    let items = r.items;
    match items {
        Some(ints) => {
            for int in ints.to_vec() {
                match int.integration_uri {
                    Some(uri) => {
                        if uri == lambda_arn {
                            return int.integration_id;
                        }
                    }
                    None => (),
                };
            }
            None
        }
        None => None,
    }
}

fn make_request_params(is_async: bool) -> Option<HashMap<String, String>> {
    if is_async {
        let mut h: HashMap<String, String> = HashMap::new();
        h.insert(
            s!("integration.request.header.X-Amz-Invocation-Type"),
            s!("'Event'"),
        );
        Some(h)
    } else {
        None
    }
}

async fn create(
    client: &Client,
    api_id: &str,
    lambda_arn: &str,
    role_arn: &str,
    is_async: bool,
) -> Result<String, Error> {
    let req_params = make_request_params(is_async);
    let res = client
        .create_integration()
        .api_id(s!(api_id))
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("2.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_uri(lambda_arn)
        .set_request_parameters(req_params)
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
    lambda_arn: &str,
    role_arn: &str,
    is_async: bool,
) -> Result<String, Error> {
    let req_params = make_request_params(is_async);
    let res = client
        .update_integration()
        .api_id(s!(api_id))
        .integration_id(id)
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("2.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_uri(lambda_arn)
        .set_request_parameters(req_params)
        .send()
        .await;
    match res {
        Ok(r) => Ok(r.integration_id.unwrap()),
        Err(e) => panic!("{:?}", e),
    }
}

pub async fn create_or_update(
    client: &Client,
    api_id: &str,
    lambda_arn: &str,
    role_arn: &str,
    is_async: bool,
) -> String {
    let maybe_int = find(client, api_id, lambda_arn).await;
    match maybe_int {
        Some(id) => {
            tracing::debug!("Found Lambda Integration {}", id);
            let _ = update(client, &id, api_id, lambda_arn, role_arn, is_async).await;
            id
        }
        _ => {
            let id = create(client, api_id, lambda_arn, role_arn, is_async)
                .await
                .unwrap();
            tracing::debug!("Created Lambda Integration {}", id);
            id
        }
    }
}

pub async fn delete(client: &Client, api_id: &str, lambda_arn: &str) {
    let maybe_int = find(client, api_id, lambda_arn).await;
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
