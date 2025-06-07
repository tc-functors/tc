use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HandlerSpec {
    #[serde(default)]
    pub handler: Option<String>,

    #[serde(default)]
    pub event: Option<String>,

    #[serde(default)]
    pub function: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelSpec {
    #[serde(default)]
    pub doc_only: bool,
    pub on_publish: Option<HandlerSpec>,
    pub on_subscribe: Option<HandlerSpec>,
}
