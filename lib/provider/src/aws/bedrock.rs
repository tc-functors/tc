use crate::Auth;

use aws_sdk_bedrockruntime::{
    types::{ContentBlock, ConversationRole, Message},
    types::InferenceConfiguration,
    Client,
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

    let response = client
        .converse()
        .model_id(model)
        .messages(message)
        .inference_config(
            InferenceConfiguration::builder()
                .max_tokens(20000)
                .temperature(0.7)
                .build()
        )
        .send()
        .await
        .unwrap();

    response
        .output()
        .unwrap()
        .as_message()
        .unwrap()
        .content()
        .first()
        .unwrap()
        .as_text()
        .unwrap()
        .to_string()
}
