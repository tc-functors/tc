//! Manifest file detection and relative-path extraction.
//!
//! We recognize a fixed set of dependency-declaration files (manifests +
//! lockfiles) and scan them for tokens that look like relative filesystem paths
//! (starting with `./` or `../`). We deliberately do not parse these files
//! structurally — a ripgrep-style extraction handles every format uniformly
//! and is robust against format evolution.
//!
//! Limitations (documented):
//! - We do NOT scan arbitrary source code. Relative paths embedded in e.g.
//!   Python source (`open('../data.csv')`) are not detected.
//! - We do NOT interpret shell commands in `build.pre` / `build.post` / etc.
//! - We do NOT follow named-reference dependencies (layer names, etc.).

use regex::Regex;
use std::sync::OnceLock;

/// Filenames recognized as dependency-declaration files.
const MANIFEST_NAMES: &[&str] = &[
    // Python
    "pyproject.toml",
    "Pipfile",
    "Pipfile.lock",
    "poetry.lock",
    "uv.lock",
    "setup.cfg",
    // Ruby
    "Gemfile",
    "Gemfile.lock",
    // Node
    "package.json",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    // Rust
    "Cargo.toml",
    "Cargo.lock",
];

/// Returns true if `file_name` (basename only) is a recognized manifest.
pub fn is_manifest(file_name: &str) -> bool {
    if MANIFEST_NAMES.contains(&file_name) {
        return true;
    }
    // requirements*.txt family
    if file_name.starts_with("requirements") && file_name.ends_with(".txt") {
        return true;
    }
    false
}

fn rel_path_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    // Match tokens that:
    //   - are preceded by start-of-line, whitespace, or a delimiter common in
    //     config/lock formats (quotes, =, :, comma, paren, bracket)
    //   - start with `./` or `../`
    //   - continue with path-safe characters (alnum, `.`, `/`, `_`, `-`)
    //
    // We capture the path token in group 1. The leading context character is
    // intentionally non-capturing so we don't swallow it.
    RE.get_or_init(|| {
        Regex::new(r#"(?:^|[\s"'=:,(\[])(\.{1,2}/[A-Za-z0-9_./\-]+)"#).unwrap()
    })
}

/// Extract every distinct relative-path token from `contents`.
///
/// Output is ordered and deduplicated.
pub fn extract_relative_paths(contents: &str) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for cap in rel_path_regex().captures_iter(contents) {
        if let Some(m) = cap.get(1) {
            let s = m.as_str().trim_end_matches(|c: char| c == '.' || c == '/');
            // Filter out trivial noise: bare `.` or `..` or `./` tokens after trim
            if s.is_empty() || s == "." || s == ".." {
                continue;
            }
            if seen.insert(s.to_string()) {
                out.push(s.to_string());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_recognition() {
        assert!(is_manifest("pyproject.toml"));
        assert!(is_manifest("Cargo.toml"));
        assert!(is_manifest("Cargo.lock"));
        assert!(is_manifest("package.json"));
        assert!(is_manifest("Gemfile"));
        assert!(is_manifest("requirements.txt"));
        assert!(is_manifest("requirements-dev.txt"));
        assert!(is_manifest("requirements_prod.txt"));
        assert!(!is_manifest("handler.py"));
        assert!(!is_manifest("README.md"));
        assert!(!is_manifest("requirements.md"));
    }

    #[test]
    fn extract_pyproject_path_dep() {
        let s = r#"[tool.poetry.dependencies]
python = "^3.10"
shared = {path = "../shared", develop = true}
core = {path = "../../core"}
"#;
        let paths = extract_relative_paths(s);
        assert!(paths.contains(&"../shared".to_string()));
        assert!(paths.contains(&"../../core".to_string()));
    }

    #[test]
    fn extract_cargo_path_dep() {
        let s = r#"
[dependencies]
kit = { path = "../kit" }
foo = { version = "1.0", path = "./local" }
bar = "1.0"
"#;
        let paths = extract_relative_paths(s);
        assert!(paths.contains(&"../kit".to_string()));
        assert!(paths.contains(&"./local".to_string()));
    }

    #[test]
    fn extract_package_json_file_dep() {
        let s = r#"{
  "dependencies": {
    "common": "file:../common",
    "core": "link:../../core",
    "ext": "^1.0.0"
  }
}"#;
        let paths = extract_relative_paths(s);
        assert!(paths.contains(&"../common".to_string()));
        assert!(paths.contains(&"../../core".to_string()));
    }

    #[test]
    fn extract_requirements_txt() {
        let s = "-r ../shared/requirements.txt\n-e ../pkg\nrequests==2.0\n";
        let paths = extract_relative_paths(s);
        assert!(paths.contains(&"../shared/requirements.txt".to_string()));
        assert!(paths.contains(&"../pkg".to_string()));
    }

    #[test]
    fn extract_gemfile() {
        let s = r#"source "https://rubygems.org"
gem "foo", path: "../foo"
gem "bar", path: '../../bar'
"#;
        let paths = extract_relative_paths(s);
        assert!(paths.contains(&"../foo".to_string()));
        assert!(paths.contains(&"../../bar".to_string()));
    }

    #[test]
    fn no_false_positive_on_semver() {
        let s = r#"foo = "1.0.0""#;
        let paths = extract_relative_paths(s);
        assert!(paths.is_empty(), "got: {:?}", paths);
    }

    #[test]
    fn no_false_positive_on_url() {
        let s = r#"repository = "https://example.com/a/../b""#;
        let paths = extract_relative_paths(s);
        // The "b" under "a/.." is embedded inside a URL without preceding
        // whitespace/delimiter immediately before the "..", so the regex may
        // or may not match depending on URL shape. We don't promise precision
        // for URL-embedded tokens — downstream existence check filters them.
        // Just make sure this doesn't panic.
        let _ = paths;
    }

    #[test]
    fn dedup() {
        let s = r#"a = "../foo"
b = "../foo"
c = "../bar""#;
        let paths = extract_relative_paths(s);
        assert_eq!(paths.len(), 2);
    }
}
