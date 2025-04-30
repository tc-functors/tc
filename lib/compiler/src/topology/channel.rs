use super::template;
use crate::{
    spec::{
        ChannelSpec,
        HandlerSpec,
    },
};
use kit as u;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

fn default_handler() -> String {
    format!(
        r#"export function onSubscribe(ctx) {{
       return ctx.events
}}

export function onPublish(ctx) {{
  return ctx.events }}
"#
    )
}

fn event_handler(_event_name: &str) -> String {
    default_handler()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Channel {
    pub handler: String,
    pub name: String,
    pub api_name: String,
}

fn find_handler(hs: &HandlerSpec) -> String {
    println!("{:?}", &hs);
    let HandlerSpec { handler, event, .. } = hs;
    if let Some(h) = handler {
        match h.as_ref() {
            "default" => default_handler(),
            _ => u::slurp(&h),
        }
    } else {
        match event {
            Some(e) => event_handler(&e),
            None => default_handler(),
        }
    }
}

pub fn make(namespace: &str, spec: HashMap<String, ChannelSpec>) -> HashMap<String, Channel> {
    let mut h: HashMap<String, Channel> = HashMap::new();
    for (name, s) in spec {
        let handler = match &s.on_publish {
            Some(hs) => find_handler(hs),
            None => default_handler(),
        };

        let c = Channel {
            name: name.clone(),
            handler: handler,
            api_name: template::topology_fqn(namespace, false),
        };
        h.insert(name, c);
    }
    h
}
