use super::index;
use kit as u;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Hook {
    pub command: String,
    #[serde(default)]
    pub on_failure: Option<String>,
    #[serde(default)]
    pub dir: Option<String>,
}

pub fn load(infra_dir: &str) -> HashMap<String, Vec<Hook>> {
    let hooks_file = format!("{}/hooks.json", infra_dir);
    if index::get().file_exists(&hooks_file) {
        let data: String = u::slurp(&hooks_file);
        let hooks: HashMap<String, Vec<Hook>> = serde_json::from_str(&data).unwrap();
        hooks
    } else {
        HashMap::new()
    }
}
