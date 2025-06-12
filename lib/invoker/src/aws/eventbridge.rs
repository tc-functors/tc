use authorizer::Auth;

use aws_sdk_eventbridge::{
    Client,
    types::{PutEventsRequestEntry, builders::PutEventsRequestEntryBuilder},
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

fn make_event(bus: &str, detail_type: &str, source: &str, detail: &str) -> PutEventsRequestEntry {
    let e = PutEventsRequestEntryBuilder::default();
    let event = e
        .source(source)
        .detail_type(detail_type)
        .detail(detail)
        .event_bus_name(bus)
        .build();
    event
}

pub async fn put_event(client: Client, bus: &str, detail_type: &str, source: &str, detail: &str) {
    let event = make_event(bus, detail_type, source, detail);
    let resp = client.put_events().entries(event).send().await;
    println!("{:?}", resp);
}
