use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use std::process::exit;

use kit as u;
use kit::*;

fn default() -> String {
    s!("")
}

fn default_vec() -> Vec<String> {
    vec![]
}

fn default_hashmap() -> HashMap<String, String> {
    HashMap::new()
}

fn default_bool() -> bool {
    false
}

fn default_int() -> u8 {
    1
}

fn default_bus() -> String {
    s!("default")
}

fn default_ci_provider() -> String {
    s!("circecli")
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

fn default_layers_profile() -> Option<String> {
    None
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
pub struct Compiler {
    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub verify: bool,

    #[derivative(Default(value = "default_int()"))]
    #[serde(default = "default_int")]
    pub graph_depth: u8,

    #[derivative(Default(value = "default()"))]
    #[serde(default = "default")]
    pub default_infra_path: String,
}


#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Resolver {
    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub incremental: bool,

    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub cache: bool,

    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub layer_promotions: bool,

    #[derivative(Default(value = "default()"))]
    #[serde(default = "default")]
    pub stable_sandbox: String,
}


#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Deployer {
    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub guard_stable_updates: bool,

    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub rolling: bool,

    #[derivative(Default(value = "default()"))]
    #[serde(default = "default")]
    pub fallback: String,
}


#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Notifier {
    #[derivative(Default(value = "default_hashmap()"))]
    #[serde(default = "default_hashmap")]
    pub webhooks: HashMap<String, String>,
}

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Ci {
    #[derivative(Default(value = "default_ci_provider()"))]
    #[serde(default = "default_ci_provider")]
    pub provider: String,


    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub assume_role: bool,

    #[derivative(Default(value = "default_bool()"))]
    #[serde(default = "default_bool")]
    pub update_metadata: bool,

    #[derivative(Default(value = "default_hashmap()"))]
    #[serde(default = "default_hashmap")]
    pub roles: HashMap<String, String>,

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
    pub layers_profile: Option<String>,

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


#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Aws {

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

#[derive(Derivative, Serialize, Deserialize, Clone)]
#[derivative(Debug, Default)]
pub struct Config {

    #[serde(default = "Compiler::default")]
    pub compiler: Compiler,

    #[serde(default = "Resolver::default")]
    pub resolver: Resolver,

    #[serde(default = "Deployer::default")]
    pub deployer: Deployer,

    #[serde(default = "Aws::default")]
    pub aws: Aws,

    #[serde(default = "Notifier::default")]
    pub notifier: Notifier,

   #[serde(default = "Ci::default")]
    pub ci: Ci,

}


fn render_template(env: &str, cfg_str: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("env", env);
    u::stencil(cfg_str, table)
}

impl Config {
    pub fn new(path: Option<String>, env: &str) -> Config {

        let config_path = match std::env::var("TC_CONFIG_PATH") {
            Ok(p) => kit::expand_path(&p),
            Err(_) =>  match path {
                Some(p) => p,
                None => {
                    let root = kit::sh("git rev-parse --show-toplevel", &kit::pwd());
                    format!("{}/infrastructure/tc/config.yml", root)
                }
            }
        };

        match fs::read_to_string(&config_path) {
            Ok(c) => {
                let rendered = render_template(env, &c);
                let cfg: Config = match serde_yaml::from_str(&rendered) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("{:?}", e);
                        eprintln!("Unable to load data from `{}`", &config_path);
                        exit(1);
                    }
                };
                cfg
            }
            Err(_) => Config::default(),
        }
    }

    pub fn render(&self) -> String {
        kit::pretty_json(self)
    }

    pub fn notification_webhook(&self, env: &str) -> Option<String> {
        self.notifier.webhooks.get(env).cloned()
    }
}
