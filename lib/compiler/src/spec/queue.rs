use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueueSpec {
    #[serde(default)]
    pub producer: String,

    pub name: Option<String>,

    pub function: Option<String>,
}
