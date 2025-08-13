use crate::aws::eventbridge;
use authorizer::Auth;
use colored::Colorize;

pub async fn trigger(auth: &Auth, bus: &str, detail_type: &str, source: &str, detail: &str) {
    let client = eventbridge::make_client(auth).await;
    println!("Triggering event: detail-type: {} source: {}", detail_type.cyan(), source.green());
    let id = eventbridge::put_event(client, &bus, detail_type, source, detail).await;
    println!("Event id: {}", &id);
}
