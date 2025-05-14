use crate::{
    spec::TriggerSpec,
};
use super::template;
use std::collections::HashMap;

pub fn make(_namespace: &str, spec: HashMap<String, TriggerSpec>) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    for (name, s) in spec {

        let f = match s.function {
            Some(f) => f,
            None => panic!("No function specified for trigger")
        };

        let realf = if f.starts_with("{{namespace") {
            f
        } else {
            format!("{{{{namespace}}}}_{f}_{{{{sandbox}}}}")
        };

        h.insert(name, template::lambda_arn(&realf));
    }
    h
}
