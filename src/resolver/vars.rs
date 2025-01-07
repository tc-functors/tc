use aws::Env;
use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Context;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SomeVars {
    pub memory_size: Option<i32>,
    pub timeout: Option<i32>,
    pub image: Option<String>,
    pub image_uri: Option<String>,
    pub provisioned_concurrency: Option<i32>,
    pub environment: Option<HashMap<String, String>>,
    pub network: Option<HashMap<String, Vec<String>>>,
    pub tags: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vars {
    pub memory_size: i32,
    pub timeout: i32,
    pub image: String,
    pub provisioned_concurrency: Option<i32>,
    pub environment: HashMap<String, String>,
    pub network: HashMap<String, Vec<String>>,
    pub tags: HashMap<String, String>,
}

fn render_var(s: &str, env: &Env) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    let account = &env.account();
    let region = &env.region();
    table.insert("env", &env.name);
    table.insert("account", account);
    table.insert("region", region);
    u::stencil(s, table)
}

fn unwrap_vars(env: &Env, v: SomeVars) -> Vars {
    let image = u::maybe_string(v.image_uri, &u::maybe_string(v.image, ""));
    Vars {
        memory_size: u::maybe_int(v.memory_size, 128),
        timeout: u::maybe_int(v.timeout, 600),
        image: render_var(&image, env),
        provisioned_concurrency: v.provisioned_concurrency,
        environment: u::maybe_hashmap(v.environment, HashMap::new()),
        tags: u::maybe_hashmap(v.tags, HashMap::new()),
        network: match v.network {
            Some(n) => n,
            None => HashMap::new(),
        },
    }
}

type VarsMap = HashMap<String, SomeVars>;

pub fn default_vars(context: &Context, lang: &str, fqn: &str, dir: &str) -> Vars {
    let Context {
        env,
        namespace,
        sandbox,
        ..
    } = context;
    let profile = &env.name;

    let mn = u::pascal_case(&format!("{} {}", namespace, fqn));
    let mut hmap: HashMap<String, String> = HashMap::new();
    hmap.insert(String::from("LAMBDA_STAGE"), s!(profile));
    hmap.insert(String::from("Environment"), s!(profile));
    hmap.insert(String::from("SHA"), git::sha());
    hmap.insert(String::from("AWS_ACCOUNT_ID"), env.account().to_owned());
    hmap.insert(String::from("SANDBOX"), s!(sandbox));
    hmap.insert(String::from("NAMESPACE"), s!(namespace));
    hmap.insert(String::from("LOG_LEVEL"), s!("INFO"));
    hmap.insert(String::from("POWERTOOLS_METRICS_NAMESPACE"), mn);

    match lang {
        "ruby2.7" | "ruby3.2" | "ruby32" | "ruby27" => {
            hmap.insert(String::from("GEM_PATH"), "/opt/ruby/gems/3.2.0".to_string());

            hmap.insert(
                String::from("BUNDLE_CACHE_PATH"),
                "/opt/ruby/lib".to_string(),
            );

            hmap.insert(String::from("RUBYLIB"), "$RUBYLIB:/opt/lib".to_string());

            match std::env::var("NO_RUBY_WRAPPER") {
                Ok(_) => {}
                Err(_) => {
                    if u::path_exists(dir, "Gemfile") {
                        hmap.insert(
                            String::from("AWS_LAMBDA_EXEC_WRAPPER"),
                            "/opt/ruby/wrapper".to_string(),
                        );
                    }
                }
            }

            if sandbox != "stable" {
                hmap.insert(
                    String::from("HONEYBADGER_ENV"),
                    format!("{}-{}", profile, sandbox),
                );
            } else {
                hmap.insert(String::from("HONEYBADGER_ENV"), s!(profile));
            }
        }
        _ => {
            if sandbox != "stable" {
                hmap.insert(
                    String::from("HONEYBADGER_ENVIRONMENT"),
                    format!("{}-{}", profile, sandbox),
                );
            } else {
                hmap.insert(String::from("HONEYBADGER_ENVIRONMENT"), s!(profile));
            }
        }
    }

    let mut tags: HashMap<String, String> = HashMap::new();
    tags.insert(String::from("deployer"), s!("tc"));

    Vars {
        memory_size: 128,
        timeout: 300,
        image: u::empty(),
        provisioned_concurrency: None,
        environment: hmap.clone(),
        network: HashMap::new(),
        tags: tags,
    }
}

fn with_defaults(context: &Context, lang: &str, vm: VarsMap, fqn: &str, dir: &str) -> Vars {
    let Context { env, sandbox, .. } = context;

    let profile = &env.name;
    let def = default_vars(context, lang, fqn, dir);

    let mut default = match vm.get("default") {
        Some(v) => unwrap_vars(env, v.clone()),
        None => def.clone(),
    };

    let fp = match vm.get(sandbox) {
        Some(v) => unwrap_vars(env, v.clone()),
        None => match vm.get(profile) {
            Some(v) => unwrap_vars(env, v.clone()),
            None => default.clone(),
        },
    };

    default.environment.extend(def.environment);
    default.environment.extend(fp.environment);
    default.network.extend(def.network);
    default.network.extend(fp.network);

    if fp.memory_size > 128 {
        default.memory_size = fp.memory_size;
    }
    if fp.timeout != 180 {
        default.timeout = fp.timeout;
    }

    default
}

impl Vars {
    pub fn new(
        context: &Context,
        vars_file: Option<String>,
        lang: &str,
        fqn: &str,
        dir: &str,
    ) -> Vars {
        match vars_file {
            Some(f) => {
                if u::file_exists(&f) {
                    let data = u::slurp(&f);
                    let map: VarsMap = serde_json::from_str(&data).unwrap();
                    with_defaults(context, lang, map, fqn, dir)
                } else {
                    default_vars(context, lang, fqn, dir)
                }
            }
            None => default_vars(context, lang, fqn, dir),
        }
    }
}
