use crate::Auth;
use aws_sdk_bedrockruntime::{
    Client,
    types::{
        ContentBlock,
        ConversationRole,
        InferenceConfiguration,
        Message,
    },
};

pub async fn make_client(auth: &Auth) -> Client {
    let shared_config = &auth.aws_config;
    Client::new(shared_config)
}

pub async fn send(client: &Client, prompt: &str, model: &str) -> String {
    let message = Message::builder()
        .role(ConversationRole::User)
        .content(ContentBlock::Text(prompt.to_string()))
        .build()
        .unwrap();

    let tokens = match std::env::var("TC_MAX_LLM_TOKENS") {
        Ok(r) => r.trim().parse::<i32>().unwrap(),
        Err(_) => 60000,
    };

    let temperature = match std::env::var("TC_LLM_TEMPERATURE") {
        Ok(r) => match r.trim() {
            "" => None,
            r => Some(r.parse::<f32>().unwrap()),
        },
        Err(_) => None,
    };

    let mut config = InferenceConfiguration::builder().max_tokens(tokens);
    if let Some(temperature) = temperature {
        config = config.temperature(temperature);
    }

    let response = client
        .converse()
        .model_id(model)
        .messages(message)
        .inference_config(config.build())
        .send()
        .await
        .expect("Bedrock Converse request failed");

    let content = response.output().unwrap().as_message().unwrap().content();

    let mut s: String = String::from("");
    for x in content {
        let data = x.as_text().unwrap().to_string();
        s.push_str(&data)
    }
    s
}
