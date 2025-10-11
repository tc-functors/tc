use composer::Channel;
use provider::{
    Auth,
    aws::appsync,
};
use std::collections::HashMap;
use kit::*;

pub async fn create(auth: &Auth, channels: &HashMap<String, Channel>) {
    let client = appsync::make_client(&auth).await;

    for (_, channel) in channels {
        println!("Creating channel: {}", &channel.name);
        let api_id = appsync::create_events_api(&client, &channel.api_name).await;
        appsync::create_events_channel(&client, &api_id, &channel.name, &channel.handler).await;
    }
}

pub async fn delete(_auth: &Auth, _channels: &HashMap<String, Channel>) {}

pub async fn update(_auth: &Auth, _channels: &HashMap<String, Channel>, _c: &str) {}

pub async fn config(auth: &Auth, name: &str, channels: &HashMap<String, Channel>) -> HashMap<String, String> {
    let client = appsync::make_client(&auth).await;
    let maybe_creds = appsync::events::find_api_creds(&client, name).await;
    let mut channel: String = String::from("");
    for (_, c ) in channels {
        channel = c.name.clone();
    }
    match maybe_creds {
        Some(creds) => {
            let mut h: HashMap<String, String> = HashMap::new();
            h.insert(s!("API_KEY"), creds.api_key);
            h.insert(s!("HTTP_DOMAIN"), creds.http_domain);
            h.insert(s!("REALTIME_DOMAIN"), creds.realtime_domain);
            h.insert(s!("CHANNEL"), channel);
            h
        }
        _ => HashMap::new(),
    }
}
