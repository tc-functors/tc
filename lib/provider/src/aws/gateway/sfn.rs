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

async fn find(client: &Client, api_id: &str, method: &str) -> Option<String> {
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
                match int.request_parameters {
                    Some(req) => match req.get("Name") {
                        Some(name) => {
                            if name == &format!("sfn-{}", method) {
                                return int.integration_id;
                            }
                        }
                        None => (),
                    },
                    None => (),
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
    sfn_arn: &str,
    role_arn: &str,
    request_template: &str,
    method: &str,
    sync: bool
) -> Result<String, Error> {

    let mut req: HashMap<String, String> = HashMap::new();
    req.insert("StateMachineArn".to_string(), s!(sfn_arn));
    req.insert("Name".to_string(), format!("sfn-{}", method));
    req.insert("Input".to_string(), request_template.to_string());
    let subtype = if sync {
        s!("StepFunctions-StartSyncExecution")
    } else {
        s!("StepFunctions-StartExecution")
    };


    let res = client
        .create_integration()
        .api_id(s!(api_id))
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("1.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_subtype(subtype)
        .set_request_parameters(Some(req))
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
    sfn_arn: &str,
    role_arn: &str,
    request_template: &str,
    method: &str,
    sync: bool
) -> String {

    let maybe_int = find(client, api_id, method).await;
    match maybe_int {
        Some(id) => id,
        _ => create(
            client,
            api_id,
            sfn_arn,
            role_arn,
            request_template,
            method,
            sync
        ).await.unwrap(),
    }
}


pub async fn delete(
    client: &Client,
    api_id: &str,
    method: &str
) {

    let maybe_int = find(client, api_id, method).await;
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
