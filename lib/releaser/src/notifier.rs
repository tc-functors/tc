use compiler::ConfigSpec;
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Text {
    pub r#type: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub r#type: String,
    pub text: Text,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Attachment {
    pub color: String,
    pub blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RichText {
    pub text: String,
    pub blocks: Vec<Block>,
    pub attachments: Vec<Attachment>,
}

impl RichText {
    pub fn new(title: &str, summary: &str, msg: &str) -> RichText {
        let msg_block = Block {
            r#type: s!("section"),
            text: Text {
                r#type: s!("mrkdwn"),
                text: format!("```{}```", msg),
            },
        };

        let title_block = Block {
            r#type: s!("section"),
            text: Text {
                r#type: s!("mrkdwn"),
                text: format!("*{}*", title),
            },
        };
        let summary_block = Block {
            r#type: s!("section"),
            text: Text {
                r#type: s!("mrkdwn"),
                text: format!("{}", summary),
            },
        };

        let attachment = Attachment {
            color: s!("#2eb886"),
            blocks: vec![msg_block],
        };
        RichText {
            text: s!(title),
            blocks: vec![title_block, summary_block],
            attachments: vec![attachment],
        }
    }
}

fn headers() -> HashMap<String, String> {
    let mut h = HashMap::new();
    h.insert(
        s!("user-agent"),
        s!("libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2"),
    );
    h.insert(s!("content-type"), s!("application/json"));
    h
}

pub fn wrap_msg(s: &str) -> String {
    format!(r#"{{"text": "{s}"}}"#)
}

pub async fn slack(scope: &str, payload: String) {
    let config = ConfigSpec::new(None);
    let url = config.notifier.webhooks.get(scope);
    match url {
        Some(u) => {
            let _ = u::http_post(&u, headers(), payload.to_string()).await;
            //println!("{:?}", res);
        }
        None => {
            println!("No notification webhook url found");
        }
    }
}

pub async fn notify(scope: &str, msg: &str) {
    slack(scope, wrap_msg(msg)).await;
}
