use serde_derive::{
    Deserialize,
    Serialize,
};
use kit as u;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Connector {
    pub source: String,
    pub message: String,
    pub target: String
}

pub fn make_all(maybe_seq: &Option<Vec<String>>) -> Vec<Connector> {

    if let Some(cspecs) = maybe_seq {
        let mut cs: Vec<Connector> = vec![];
        for x in cspecs {
            let s = x.replace(" ", "");
            let parts: Vec<&str> = s.split("->").collect();
            let source = parts.clone().into_iter().nth(0).unwrap_or_default();
            let message = parts.clone().into_iter().nth(1).unwrap_or_default();
            let target = parts.clone().into_iter().nth(2).unwrap_or_default();
            let c = Connector {
                source: source.to_string(),
                message: message.to_string(),
                target: target.to_string()
            };
            cs.push(c);
        }
        cs
    } else {
        vec![]
    }
}
