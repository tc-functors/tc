use super::Topology;
use kit as u;
use serde_json::Value;
use std::collections::HashMap;

pub fn find_assets(dir: &str, topology: &Topology) -> HashMap<String, Value> {
    if !u::path_exists(dir, "function.json") {
        return HashMap::new();
    }
    let functions = &topology.functions;
    let mut fns = functions.get(dir).into_iter();
    if fns.len() > 0 {
        let f = fns.nth(0);
        match f {
            Some(h) => h.assets.to_owned(),
            None => HashMap::new(),
        }
    } else {
        HashMap::new()
    }
}
