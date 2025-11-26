use kit as u;
use kit::*;
use serde::{Deserialize,Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
struct Content {
    r#type: String,
    text: String,
}

#[derive(Serialize, Debug)]
struct Message {
    pub role: String,
    pub content: Vec<Content>,
}

fn headers() -> HashMap<String, String> {
    let api_key = match std::env::var("CLAUDE_API_KEY") {
        Ok(p) => p,
        Err(_) => String::from(""),
    };
    let mut h = HashMap::new();
    h.insert(s!("content-type"), s!("application/json"));
    h.insert(s!("anthropic-version"), s!("2023-06-01"));
    h.insert(s!("x-api-key"), api_key);
    h.insert(s!("accept"), s!("application/json"));
    h.insert(
        s!("user-agent"),
        s!("libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2"),
    );
    h
}

#[derive(Serialize, Debug)]
struct Payload {
    model: String,
    max_tokens: u16,
    messages: Vec<Message>,
}

impl Payload {
    fn new(text: &str, model: &str) -> Payload {
        let content = Content {
            r#type: s!("text"),
            text: s!(text),
        };

        let message = Message {
            role: s!("user"),
            content: vec![content],
        };

        Payload {
            model: s!(model),
            max_tokens: 20000,
            messages: vec![message],
        }
    }
}

#[derive(Deserialize)]
struct Response {
    content: Vec<Content>,
}

pub async fn send(text: &str, model: &str) -> String {
    let payload = Payload::new(&text, model);
    let p = serde_json::to_string(&payload).unwrap();
    let url = "https://api.anthropic.com/v1/messages";
    let res = u::http_post(url, headers(), p).await.unwrap();
    let response: Response = serde_json::from_value(res).unwrap();
    let res = response.content.into_iter().nth(0).unwrap().text;
    res
}
