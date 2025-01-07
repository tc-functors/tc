use serde_derive::{Deserialize, Serialize};
use std::env;

pub fn as_sandbox(sandbox: Option<String>) -> String {
    match sandbox {
        Some(s) => s,
        None => match env::var("TC_SANDBOX") {
            Ok(s) => s,
            Err(_) => git::branch_name(),
        },
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sandbox {
    pub name: String,
}

impl Sandbox {
    pub fn new(name: Option<String>) -> Sandbox {
        Sandbox {
            name: as_sandbox(name),
        }
    }
}
