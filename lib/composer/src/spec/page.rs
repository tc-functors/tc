use serde_derive::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PageSpec {
    pub dist: Option<String>,
    pub build: Option<Vec<String>>,
    pub dir: Option<String>,
    pub domains: Option<Vec<String>>,
    pub paths: Option<Vec<String>>,
    pub bucket: Option<String>,
    pub config: Option<String>,
}
