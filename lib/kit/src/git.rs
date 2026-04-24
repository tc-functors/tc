use crate::{
    pwd,
    sh,
};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

fn extract_version(s: &str) -> String {
    let re: Regex = Regex::new(r"(?:(\d+)\.)?(?:(\d+)\.)?(?:(\d+)\.\d+)").unwrap();
    let matches = re.find(s);
    match matches {
        Some(m) => m.as_str().to_string(),
        _ => "0.0.2".to_string(),
    }
}

/// Returns the most recent tag matching `{prefix}-*` reachable from HEAD,
/// stripped to a semver string. Results are cached for the lifetime of the
/// process (git state is immutable during a single CLI invocation), so
/// callers iterating over many namespaces pay for each unique prefix
/// exactly once instead of forking `git describe` per call site.
pub fn current_semver(prefix: &str) -> String {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock() {
        if let Some(cached) = guard.get(prefix) {
            return cached.clone();
        }
    }
    let cmd = format!(
        "git describe --match {}-* --tags $(git log -n1 --pretty='%h')",
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
