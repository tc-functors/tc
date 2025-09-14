use crate::aws::eventbridge;
use provider::Auth;
use colored::Colorize;
use composer::Event;

pub async fn trigger(auth: &Auth, event: &Event, payload: &str) {
    let Event { pattern, bus, .. } = event;

    let detail_type = pattern.detail_type.first().unwrap();
    let source = pattern.source.first().unwrap();
    let detail = payload;

    let client = eventbridge::make_client(auth).await;
    println!(
        "Triggering event: detail-type: {} source: {}",
        detail_type.cyan(),
        source.green()
    );
    let id = eventbridge::put_event(client, bus, detail_type, source, detail).await;
    println!("Event id: {}", &id);
}
