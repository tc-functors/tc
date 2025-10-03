use super::template;
use compiler::{
    Entity,
    spec::{
        TableSpec,
    },
};
use kit as u;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Table {
    pub handler: String,
    pub name: String,
    pub api_name: String,
    pub targets: Vec<Target>,
}
