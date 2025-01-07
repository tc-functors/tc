use colored_json::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

pub fn json_value(s: &str) -> Value {
    let json: Value = serde_json::from_str(s).expect("fail");
    json
}

pub fn json_value_safe(s: &str) -> Value {
    let m = serde_json::from_str(s);
    match m {
        Ok(v) => v,
        Err(_) => "nothing".into(),
    }
}

pub fn json_map(s: &str) -> HashMap<String, String> {
    let json: HashMap<String, String> = serde_json::from_str(s).expect("fail");
    json
}
pub fn merge_json(s: &str, fields: &HashMap<String, String>) -> String {
    let v: Value = serde_json::from_str(s).expect("fail");
    let merged = match v {
        Value::Object(m) => {
            let mut m = m.clone();
            for (k, v) in fields {
                m.insert(k.clone(), Value::String(v.clone()));
            }
            Value::Object(m)
        }
        v => v.clone(),
    };
    serde_json::to_string(&merged).unwrap()
}

pub fn pretty_json<T: std::fmt::Debug + Serialize>(x: T) -> String {
    serde_json::to_string_pretty(&x)
        .unwrap()
        .to_colored_json_auto()
        .unwrap()
}

pub fn json_to_string<T: std::fmt::Debug + Serialize>(x: T) -> String {
    serde_json::to_string(&x).unwrap()
}

pub fn json_to_hashmap(json: &str, keys: Vec<&str>) -> HashMap<String, Value> {
    let mut lookup: HashMap<String, Value> = serde_json::from_str(json).unwrap();
    let mut map = HashMap::new();
    for key in keys {
        let (k, v) = lookup.remove_entry(key).unwrap();
        map.insert(k, v);
    }
    map
}

pub fn value_to_string(val: Option<&Value>) -> String {
    match val {
        Some(v) => v.as_str().unwrap().to_string(),
        None => String::from(""),
    }
}
