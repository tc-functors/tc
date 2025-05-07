use doku::Document;
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashMap,
};

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct NetworkSpec {
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct FilesystemSpec {
    pub arn: String,
    pub mount_point: String,
}

fn default_memory_size() -> Option<i32> {
    Some(128)
}

fn default_timeout() -> Option<i32> {
    Some(300)
}

#[derive(Serialize, Deserialize, Clone, Debug, Document)]
pub struct InfraSpec {
    #[serde(default = "default_memory_size")]
    pub memory_size: Option<i32>,
    #[serde(default = "default_timeout")]
    pub timeout: Option<i32>,
    pub image_uri: Option<String>,
    pub provisioned_concurrency: Option<i32>,
    pub reserved_concurrency: Option<i32>,
    pub environment: Option<HashMap<String, String>>,
    pub network: Option<NetworkSpec>,
    pub filesystem: Option<FilesystemSpec>,
    pub tags: Option<HashMap<String, String>>,
}

impl InfraSpec {
    pub fn new(runtime_file: Option<String>) -> HashMap<String, InfraSpec> {
        match runtime_file {
            Some(f) => {
                let data = u::slurp(&f);
                let ris: HashMap<String, InfraSpec> = serde_json::from_str(&data).unwrap();
                ris
            }
            None => {
                let mut h: HashMap<String, InfraSpec> = HashMap::new();
                let r = InfraSpec {
                    memory_size: Some(128),
                    timeout: Some(300),
                    image_uri: None,
                    provisioned_concurrency: None,
                    reserved_concurrency: None,
                    environment: None,
                    network: None,
                    filesystem: None,
                    tags: None,
                };
                h.insert(s!("default"), r);
                h
            }
        }
    }
}
