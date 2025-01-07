use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::exit;
use toml;

use kit::*;

fn default() -> String {
    s!("")
}

fn default_vec() -> Vec<String> {
    vec![]
}

fn default_bus() -> String {
    s!("default")
}

fn default_rule_prefix() -> String {
    s!("tc-")
}

fn default_event_role() -> String {
    s!("tc-event-base-role")
}

fn default_lambda_role() -> String {
    s!("tc-lambda-base-role")
}

fn default_sfn_role() -> String {
    s!("tc-sfn-base-role")
}

fn default_timeout() -> u8 {
    180
}

fn default_layers_profile() -> String {
    s!("dev")
}

fn default_region() -> String {
    s!("us-west-2")
}

fn default_api_name() -> String {
    s!("us-west-2")
}

fn default_mountpoint() -> String {
    s!("/mnt/assets")
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct General {
    #[serde(default = "default")]
    pub assume_role: String,
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Eventbridge {
    #[derivative(Default(value = "default_bus()"))]
    #[serde(default = "default_bus")]
    pub bus: String,

    #[derivative(Default(value = "default_rule_prefix()"))]
    #[serde(default = "default_rule_prefix")]
    pub rule_prefix: String,

    #[derivative(Default(value = "default_event_role()"))]
    #[serde(default = "default_event_role")]
    pub default_role: String,

    #[derivative(Default(value = "default_region()"))]
    #[serde(default = "default_region")]
    pub default_region: String,
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Efs {
    #[derivative(Default(value = "default()"))]
    #[serde(default)]
    pub subnets: String,

    #[derivative(Default(value = "default()"))]
    #[serde(default)]
    pub security_group: String,

    #[derivative(Default(value = "default()"))]
    #[serde(default)]
    pub dev_ap: String,

    #[derivative(Default(value = "default()"))]
    #[serde(default)]
    pub stable_ap: String,

    #[derivative(Default(value = "default_region()"))]
    #[serde(default = "default_region")]
    pub default_region: String,
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Ecs {
    #[derivative(Default(value = "default_vec()"))]
    #[serde(default)]
    pub subnets: Vec<String>,

    #[derivative(Default(value = "default()"))]
    #[serde(default)]
    pub cluster: String,
}


#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Stepfunction {
    #[derivative(Default(value = "default_sfn_role()"))]
    #[serde(default = "default_sfn_role")]
    pub default_role: String,

    #[derivative(Default(value = "default_region()"))]
    #[serde(default = "default_region")]
    pub default_region: String,
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Lambda {
    #[derivative(Default(value = "default_timeout()"))]
    #[serde(default = "default_timeout")]
    pub default_timeout: u8,

    #[derivative(Default(value = "default_lambda_role()"))]
    #[serde(default = "default_lambda_role")]
    pub default_role: String,

    #[derivative(Default(value = "default_region()"))]
    #[serde(default = "default_region")]
    pub default_region: String,

    #[derivative(Default(value = "default_layers_profile()"))]
    #[serde(default = "default_layers_profile")]
    pub layers_profile: String,

    #[derivative(Default(value = "default_mountpoint()"))]
    #[serde(default = "default_mountpoint")]
    pub fs_mountpoint: String,
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct ApiGateway {
    #[derivative(Default(value = "default_api_name()"))]
    #[serde(default = "default_api_name")]
    pub api_name: String,

    #[derivative(Default(value = "default_region()"))]
    #[serde(default = "default_region")]
    pub default_region: String,
}

// struct defaults

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Config {
    #[serde(default = "General::default")]
    pub general: General,

    #[serde(default = "Eventbridge::default")]
    pub eventbridge: Eventbridge,

    #[serde(default = "Efs::default")]
    pub efs: Efs,

    #[serde(default = "Ecs::default")]
    pub ecs: Ecs,

    #[serde(default = "Stepfunction::default")]
    pub stepfunction: Stepfunction,

    #[serde(default = "Lambda::default")]
    pub lambda: Lambda,

    #[serde(default = "ApiGateway::default")]
    pub api_gateway: ApiGateway,
}

impl Config {
    pub fn new() -> Config {
        // look for possible paths

        let path = match std::env::var("TC_CONFIG_PATH") {
            Ok(p) => kit::expand_path(&p),
            Err(_) => {
                let root = kit::sh("git rev-parse --show-toplevel", &kit::pwd());
                format!("{}/infrastructure/tc/config.toml", root)

            }
        };
        println!("Loading config {}", &path);

        match fs::read_to_string(&path) {
            Ok(c) => {
                let cfg: Config = match toml::from_str(&c) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("{:?}", e);
                        eprintln!("Unable to load data from `{}`", path);
                        exit(1);
                    }
                };
                cfg
            }
            Err(_) => Config::default(),
        }
    }
}
