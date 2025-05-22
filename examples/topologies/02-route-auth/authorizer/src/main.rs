use std::collections::HashMap;
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::Value;
use serde_json::json;
use serde_derive::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

mod policy;
use policy::{Policy, AuthorizerPolicy};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct RequestContext {
    account_id: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthorizerRequest {
    #[serde(rename = "type")]
    _type: String,
    route_arn: String,
    route_key: String,
    raw_path: String,
    headers: HashMap<String, String>,
    request_context: RequestContext
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AuthorizerResponse {
    principal_id: String,
    policy_document: AuthorizerPolicy,
    context: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct Creds {
    auth_type: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>
}

fn decode_creds(headers: HashMap<String, String>) -> Creds {
    let authorization = headers.get("authorization").unwrap();
    let auth_tmp: Vec<&str> = authorization.split(" ").collect();
    let auth_type = auth_tmp[0];
    let token = auth_tmp[1];

    match auth_type {
        "Basic" => {
            let decoded_bytes = general_purpose::STANDARD_NO_PAD
                .decode(token)
                .unwrap();
            let decoded = std::str::from_utf8(&decoded_bytes).unwrap();

            let auth_tmp: Vec<&str> = decoded.split(":").collect();
            Creds {
                auth_type: String::from("Basic"),
                username: Some(auth_tmp[0].to_string()),
                password: Some(auth_tmp[1].to_string()),
                token: None
            }
        },
        "Bearer" => {
            Creds {
                auth_type: String::from("JWT"),
                username: None,
                password: None,
                token: Some(token.to_string())
            }
        }
        &_ => todo!()
    }
}

async fn validate(_creds: Creds) -> bool {
    // lookup db or cache
    // this is the only domain-specifc lookup and could be configurable
    true
}

async fn authorize(req: AuthorizerRequest) -> AuthorizerPolicy {
    let AuthorizerRequest { route_arn, headers, .. } = req;

    let tmp: Vec<&str> = route_arn.split(":").collect();
    let aws_account_id = tmp[4];
    let api_gateway_arn_tmp: Vec<&str> = tmp[5].split("/").collect();
    let account_id = tmp[4];
    let region = tmp[3];
    let rest_api_id = api_gateway_arn_tmp[0];
    let stage = api_gateway_arn_tmp[1];

    //let creds = decode_creds(headers);
    //let is_valid = validate(creds).await;
    let is_valid = true;

    let policy;
    if is_valid {
        policy = Policy::new(region, account_id, rest_api_id, stage)
            .allow_all_methods()
            .build();
    } else {
        policy = Policy::new(region, aws_account_id, rest_api_id, stage)
            .deny_all_methods()
            .build()
    }
    policy
}

async fn handle(event: LambdaEvent<AuthorizerRequest>) -> Result<AuthorizerResponse, Error> {

    tracing::info!("request {:?}", event);
    let (authreq, _context) = event.into_parts();
    let policy = authorize(authreq.clone()).await;
    let principal_id = authreq.request_context.account_id;

    let response = AuthorizerResponse {
        principal_id: principal_id.to_string(),
        policy_document: policy,
        context: json!({
            "stringKey": "stringval",
            "numberKey": 123,
            "booleanKey": true
        }),
    };
    tracing::info!("response {:?}", response);
    Ok(response)

}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::tracing::init_default_subscriber();

    let func = service_fn(handle);
    lambda_runtime::run(func).await?;
    Ok(())
}
