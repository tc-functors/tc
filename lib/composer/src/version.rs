use kit::{
    pwd,
    sh,
};
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{
        Mutex,
        OnceLock,
    },
};

fn extract_version(s: &str) -> String {
    let re: Regex = Regex::new(r"(?:(\d+)\.)?(?:(\d+)\.)?(?:(\d+)\.\d+)").unwrap();
    let matches = re.find(s);
    match matches {
        Some(m) => m.as_str().to_string(),
        _ => "0.0.2".to_string(),
    }
}

/// Returns the most recent tag matching `{prefix}-N.N.N` reachable from
/// HEAD. Cached per process (git state doesn't move under us during one
/// CLI invocation), so 200+ functions in a topology each calling this
/// pay for each unique prefix exactly once. Mirrors `kit::current_semver`
/// but with a stricter match pattern that filters tags to semver shape.
pub fn current_semver(prefix: &str) -> String {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(prefix) {
            return cached.clone();
        }
    }
    let cmd = format!(
        "git describe --match {}-[0-9]*.[0-9]*.[0-9]* --tags $(git log -n1 --pretty='%h')",
        prefix
    );
    let out = sh(&cmd, &pwd());
    let v = if out.contains("fatal") {
        String::from("0.0.1")
    } else {
        extract_version(&out)
    };
    if let Ok(mut guard) = cache.lock() {
        guard.insert(prefix.to_string(), v.clone());
    }
    v
}
