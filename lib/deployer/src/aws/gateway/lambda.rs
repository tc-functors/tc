use aws_sdk_apigatewayv2::{
    Client,
    Error,
    types::{
        ConnectionType,
        IntegrationType,
    },
};

use kit::*;

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

async fn create(
    client: &Client,
    api_id: &str,
    lambda_arn: &str,
    role_arn: &str
) -> Result<String, Error> {

    let res = client
        .create_integration()
        .api_id(s!(api_id))
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("2.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_uri(lambda_arn)
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
    lambda_arn: &str,
    role_arn: &str,

) -> String {

    let maybe_int = find(client, api_id, lambda_arn).await;
    match maybe_int {
        Some(id) => {
            tracing::debug!("Found Lambda Integration {}", id);
            id
        }
        _ => {
            let id = create(client, api_id, lambda_arn, role_arn)
                .await
                .unwrap();
            tracing::debug!("Created Lambda Integration {}", id);
            id
        }
    }
}

pub async fn delete(
    client: &Client,
    api_id: &str,
    lambda_arn: &str,
) {

    let maybe_int = find(client, api_id, lambda_arn).await;
    match maybe_int {
        Some(id) => {
            let _ = client
                .delete_integration()
                .api_id(s!(api_id))
                .integration_id(id)
                .send()
                .await;
        },
        _ => ()
    }
}
