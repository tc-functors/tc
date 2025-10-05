use serde::{
    Deserialize,
    Serialize,
};
use std::process::exit;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StoreMode {
    Mem,
    Network,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub store_mode: StoreMode,
    pub host: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub profiles: Option<Vec<String>>,
}

impl Config {
    pub fn new(maybe_path: Option<String>) -> Config {
        match maybe_path {
            Some(path) => {
                let s = std::fs::read_to_string(&path).unwrap();
                let cfg: Config = match serde_yaml::from_str(&s) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("{:?}", e);
                        eprintln!("Unable to load data from `{}`", &path);
                        exit(1);
                    }
                };
                cfg
            }
            None => Config {
                store_mode: StoreMode::Mem,
                host: None,
                user: None,
                password: None,
                profiles: None,
            },
        }
    }
}
