use authorizer::Auth;

use crate::aws::eventbridge;

use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

// eventbridge
#[derive(Serialize, Deserialize, Debug)]
struct EventPayload {
    #[serde(rename(deserialize = "detail-type"))]
    detail_type: String,
    source: String,
    detail: Value,
}

pub async fn trigger(auth: &Auth, payload: &str) {
    let client = eventbridge::make_client(auth).await;

    let input: EventPayload = serde_json::from_str(payload).unwrap();

    println!("{:?}", input);

    let bus = "default";
    let detail = &input.detail.to_string();
    eventbridge::put_event(client, &bus, &input.detail_type, &input.source, detail).await;
}
