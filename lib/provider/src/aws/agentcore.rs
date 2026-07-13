use crate::Auth;
use aws_sdk_bedrockagentcore::{
    Client,
    primitives::Blob,
};
use serde_derive::{
    Deserialize,
    Serialize,
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn stop_runtime_session(client: &Client, runtime_arn: &str, session_id: &str) {
    let _ = client
        .stop_runtime_session()
        .agent_runtime_arn(runtime_arn)
        .runtime_session_id(session_id)
        .send()
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Payload {
    prompt: String,
}

pub async fn invoke(client: &Client, runtime_arn: &str, session_id: &str, prompt: &str) -> String {
    let payload = Payload {
        prompt: prompt.to_string(),
    };
    let payload = serde_json::to_string(&payload).unwrap();
    let blob = Blob::new(payload);
    let res = client
        .invoke_agent_runtime()
        .agent_runtime_arn(runtime_arn)
        .runtime_session_id(session_id)
        .payload(blob)
        .send()
        .await
        .unwrap();
    let bytes = res.response.collect().await.unwrap().into_bytes();
    match str::from_utf8(&bytes) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}
