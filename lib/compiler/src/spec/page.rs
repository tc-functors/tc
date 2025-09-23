use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PageSpec {
    pub dist: Option<String>,
    pub build: Option<Vec<String>>,
    pub dir: Option<String>,
    pub domains: Option<HashMap<String, String>>,
    pub paths: Option<Vec<String>>,
    pub bucket: Option<String>,
    pub config_template: Option<String>,
}
