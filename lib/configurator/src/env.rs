use kit as u;
use kit::*;
use std::{
    collections::HashMap,
    sync::OnceLock,
};

static ENV: OnceLock<HashMap<String, String>> = OnceLock::new();

fn as_defaults(xs: Vec<String>) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    for x in xs {
        h.insert(x.to_string(), "false".to_string());
    }
    h
}

fn defaults() -> HashMap<String, String> {
    as_defaults(v![
        "TC_TEST_BUCKET",
        "TC_MAX_LLM_TOKENS",
        "TC_ASSUME_ROLE",
        "TC_SFN_LOG_LEVEL",
        "TC_ECR_REPO",
        "TC_BUILD_CACHE_CLEAN",
        "TC_IMAGE_URI",
        "TC_USE_ASSET_STORE",
        "TC_SKIP_BUILD",
        "TC_USE_ASSET_STORE",
        "TC_FORCE_PUBLISH",
        "TC_INSPECT_BUILD",
        "TC_USE_CODEARTIFACT",
        "TC_SPEC_SIMPLE",
        "TC_TRACE",
        "TC_FORCE_LATEST_VERSION",
        "TC_ASSET_BUCKET",
        "TC_ASSET_ACCOUNT",
        "TC_CREATE_EVENTS",
        "TC_RESOLVE_CONCURRENCY",
        "TC_USE_STABLE_LAYERS",
        "TC_FORCE_DEPLOY",
        "TC_PRUNE_EVENT_RULES",
        "TC_SANDBOXED_EVENTS",
        "TC_LAMBDA_RUNTIME_VERSION",
        "TC_PAGES_BUCKET",
        "TC_CUSTOM_CACHE_POLICY",
        "TC_CREATE_FS_BUCKET",
        "TC_DEPLOYER_CONCURRENCY",
        "TC_DISABLE_ROUTE_TRIM",
        "TC_UPDATE_DNS",
        "TC_UPDATE_BASE_ROLES",
        "TC_VERSION_IMAGES"
    ])
}

fn load(vars: HashMap<String, String>) {
    let mut h: HashMap<String, String> = HashMap::new();
    for (k, v) in vars {
        if (v != "false") && (v != "0") {
            h.insert(k, v);
        }
    }
    let _ = ENV.set(h);
}

fn as_path(dir: &str, name: Option<String>) -> String {
    if let Some(n) = name {
        let f = format!("{}/env-{}", dir, &n);
        if u::file_exists(&f) {
            return f;
        }
    }
    if u::path_exists(dir, ".env") {
        format!("{}/.env", dir)
    } else {
        format!("{}/env-default", dir)
    }
}

pub fn init(dir: &str, name: Option<String>) {
    let path = as_path(dir, name);
    let parent_dir = u::parent_dir(&path);
    let default_path = format!("{}/env-default", &parent_dir);

    let mut gvars: HashMap<String, String> = defaults();

    if u::file_exists(&default_path) {
        dotenvy::from_path(default_path).unwrap();
        let vars: HashMap<String, String> = std::env::vars().collect();
        gvars.extend(vars)
    }

    if u::file_exists(&path) {
        dotenvy::from_path(&path).unwrap();
        let vars: HashMap<String, String> = std::env::vars().collect();
        gvars.extend(vars)
    }
    load(gvars);
}

pub fn get(key: &str) -> Option<String> {
    ENV.get()?.get(key).cloned()
}

pub fn get_all() -> HashMap<String, String> {
    ENV.get().unwrap().clone()
}
