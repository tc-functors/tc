use compiler::Channel;
use aws::appsync;
use aws::Env;
use std::collections::HashMap;

pub async fn create(env: &Env, channels: &HashMap<String, Channel>) {
    let client = appsync::make_client(&env).await;

    for (_, channel) in channels {
        println!("Creating channel: {}", &channel.name);
        let api_id = appsync::create_events_api(&client, &channel.api_name).await;
        appsync::create_events_channel(&client, &api_id, &channel.name, &channel.handler).await;
    }
}

pub async fn delete(_env: &Env, _channels: &HashMap<String, Channel>) {

}
