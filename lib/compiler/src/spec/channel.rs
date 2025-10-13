use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HandlerSpec {
    #[serde(default, alias = "function")]
    pub handler: Option<String>,

    #[serde(default)]
    pub event: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelSpec {
    #[serde(default)]
    pub doc_only: bool,
    pub function: Option<String>,
    pub on_publish: Option<HandlerSpec>,
    pub on_subscribe: Option<HandlerSpec>,
}
