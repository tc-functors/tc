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
use configurator as c;

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

    let tokens = match c::getenv("TC_MAX_LLM_TOKENS") {
        Ok(r) => r.trim().parse::<i32>().unwrap(),
        Err(_) => 60000
    };

    let response = client
        .converse()
        .model_id(model)
        .messages(message)
        .inference_config(
            InferenceConfiguration::builder()
                .max_tokens(tokens)
                .build(),
        )
        .send()
        .await
        .unwrap();

    let content = response
        .output()
        .unwrap()
        .as_message()
        .unwrap()
        .content();

    let mut s: String = String::from("");

    for x in content {
        match x.as_text() {
            Ok(r) => s.push_str(&r),
            Err(err) => panic!("{:?}", &err)

        }
    }
    s
}
