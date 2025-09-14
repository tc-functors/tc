use super::template;
use crate::spec::{
    TriggerSpec,
};
use configurator::Config;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pool {
    pub name: String,
    pub from_email: String,
    pub triggers: HashMap<String, String>,
}

pub fn make_triggers(spec: HashMap<String, TriggerSpec>) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    for (name, s) in spec {
        let f = match s.function {
            Some(f) => f,
            None => panic!("No function specified for trigger"),
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

pub fn make(
    pools: Vec<String>,
    spec: HashMap<String, TriggerSpec>,
    config: &Config,
) -> HashMap<String, Pool> {
    let mut h: HashMap<String, Pool> = HashMap::new();
    for pool in pools {
        let p = Pool {
            name: pool.clone(),
            from_email: config.aws.cognito.from_email_address.clone(),
            triggers: make_triggers(spec.clone()),
        };
        h.insert(pool, p);
    }
    h
}
