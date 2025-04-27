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

async fn create(client: &Client, api_id: &str, sfn_arn: &str, role_arn: &str, method: &str) -> Result<String, Error> {
    let mut req: HashMap<String, String> = HashMap::new();
    req.insert("StateMachineArn".to_string(), s!(sfn_arn));
    req.insert("Name".to_string(), format!("sfn-{}", method));
    if method == "POST" {
        req.insert("Input".to_string(), "{\"path\": \"${request.path}\", \"detail\": ${request.body.detail}, \"method\": \"${context.httpMethod}\"}".to_string());
    } else {
        req.insert(
            "Input".to_string(),
            "{\"path\": \"${request.path}\", \"method\": \"${context.httpMethod}\"}"
                .to_string(),
        );
    }

    let res = client
        .create_integration()
        .api_id(s!(api_id))
        .connection_type(ConnectionType::Internet)
        .credentials_arn(s!(role_arn))
        .payload_format_version(s!("1.0"))
        .integration_type(IntegrationType::AwsProxy)
        .integration_subtype(s!("StepFunctions-StartExecution"))
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
    role_arn: &str
) -> String {

    let method = "POST";
    let maybe_int = find(client, api_id, method).await;
    match maybe_int {
        Some(id) => id,
        _ => create(client, api_id, sfn_arn, role_arn, method).await.unwrap(),
    }
}
