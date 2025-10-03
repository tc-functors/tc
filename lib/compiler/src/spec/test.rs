use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TestSpec {
    #[serde(default)]
    pub payload: Option<String>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub namespace: Option<String>,

    #[serde(default)]
    pub expect: Option<String>,

    #[serde(default)]
    pub condition: Option<String>,

    #[serde(default)]
    pub auth: Option<String>,

    #[serde(default)]
    pub entity: Option<String>,
}
