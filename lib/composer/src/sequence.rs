use serde_derive::{
    Deserialize,
    Serialize,
};

use itertools::Itertools;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Connector {
    pub source_entity: String,
    pub target_entity: String,
    pub source: String,
    pub message: String,
    pub target: String
}

pub fn make_seq(cspecs: &Vec<String>) -> Vec<Connector> {
       let mut cs: Vec<Connector> = vec![];
        for x in cspecs {
            let s = x.replace(" ", "");
            let parts: Vec<&str> = s.split("->").collect();
            let source_raw = parts.clone().into_iter().nth(0).unwrap_or_default();
            let (source, source_entity) = source_raw.split("/").collect_tuple().unwrap_or((source_raw, ""));
            let message = parts.clone().into_iter().nth(1).unwrap_or_default();
            let target_raw = parts.clone().into_iter().nth(2).unwrap_or_default();
            let (target, target_entity) = target_raw.split("/").collect_tuple().unwrap_or((target_raw, ""));
            let c = Connector {
                source: source.to_string(),
                source_entity: source_entity.to_string(),
                message: message.to_string(),
                target: target.to_string(),
                target_entity: target_entity.to_string(),
            };
            cs.push(c);
        }
        cs
}

pub fn make_all(maybe_seq: &Option<Vec<String>>) -> Vec<Connector> {
    if let Some(cspecs) = maybe_seq {
        make_seq(&cspecs)
    } else {
        vec![]
    }
}
