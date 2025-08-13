use crate::aws::eventbridge;
use authorizer::Auth;

pub async fn trigger(auth: &Auth, bus: &str, detail_type: &str, source: &str, detail: &str) {
    let client = eventbridge::make_client(auth).await;
    eventbridge::put_event(client, &bus, detail_type, source, detail).await;
}
