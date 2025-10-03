use kit as u;

use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScheduleSpec {
    pub cron: String,
    pub target: String,
    pub payload: Value,
}


pub fn make(infra_dir: &str) -> HashMap<String, ScheduleSpec> {
    let path = format!("{}/schedules.json", infra_dir);
    if u::file_exists(&path) {
        let data = u::slurp(&path);
        let scheds: HashMap<String, ScheduleSpec> = serde_json::from_str(&data).unwrap();
        scheds
    } else {
        HashMap::new()
    }
}
