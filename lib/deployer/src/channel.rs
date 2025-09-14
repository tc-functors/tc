use provider::aws::appsync;
use provider::Auth;
use composer::Channel;
use std::collections::HashMap;

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
