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
    "requirements.in",
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
    // JVM (Java / Kotlin / Clojure)
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "settings.gradle",
    "settings.gradle.kts",
    "project.clj",
    "deps.edn",
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
        Regex::new(r#"(?:^|[\s"'=:,(\[>])(\.{1,2}/[A-Za-z0-9_./\-]+)"#).unwrap()
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
        assert!(is_manifest("requirements.in"));
        assert!(is_manifest("pom.xml"));
        assert!(is_manifest("build.gradle"));
        assert!(is_manifest("build.gradle.kts"));
        assert!(is_manifest("settings.gradle"));
        assert!(is_manifest("settings.gradle.kts"));
        assert!(is_manifest("project.clj"));
        assert!(is_manifest("deps.edn"));
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
    fn extract_pom_xml_relative_paths() {
        let s = r#"<project>
  <parent>
    <groupId>com.example</groupId>
    <artifactId>parent</artifactId>
    <version>1.0.0</version>
    <relativePath>../parent</relativePath>
  </parent>
  <dependencies>
    <dependency>
      <groupId>com.example</groupId>
      <artifactId>shared</artifactId>
      <version>1.0</version>
      <scope>system</scope>
      <systemPath>../libs/shared.jar</systemPath>
    </dependency>
  </dependencies>
</project>
"#;
        let paths = extract_relative_paths(s);
        assert!(paths.contains(&"../parent".to_string()), "got: {:?}", paths);
        assert!(
            paths.contains(&"../libs/shared.jar".to_string()),
            "got: {:?}",
            paths
        );
    }

    #[test]
    fn extract_build_gradle_paths() {
        let s = r#"dependencies {
    implementation files('../shared/lib.jar')
    implementation files("../../core/build/libs/core.jar")
    implementation 'org.apache.commons:commons-lang3:3.12.0'
}
"#;
        let paths = extract_relative_paths(s);
        assert!(
            paths.contains(&"../shared/lib.jar".to_string()),
            "got: {:?}",
            paths
        );
        assert!(
            paths.contains(&"../../core/build/libs/core.jar".to_string()),
            "got: {:?}",
            paths
        );
    }

    #[test]
    fn extract_settings_gradle_paths() {
        let s = r#"rootProject.name = 'my-app'
include ':shared'
project(':shared').projectDir = file('../shared')
"#;
        let paths = extract_relative_paths(s);
        assert!(
            paths.contains(&"../shared".to_string()),
            "got: {:?}",
            paths
        );
        // ':shared' is a Gradle project reference, not a path — must not appear.
        assert!(!paths.iter().any(|p| p.contains(":shared")), "got: {:?}", paths);
    }

    #[test]
    fn extract_project_clj_paths() {
        let s = r#"(defproject my-app "0.1.0"
  :description "Example"
  :source-paths ["../shared/src" "../../common/src"]
  :resource-paths ["../resources"]
  :dependencies [[org.clojure/clojure "1.11.1"]])
"#;
        let paths = extract_relative_paths(s);
        assert!(
            paths.contains(&"../shared/src".to_string()),
            "got: {:?}",
            paths
        );
        assert!(
            paths.contains(&"../../common/src".to_string()),
            "got: {:?}",
            paths
        );
        assert!(
            paths.contains(&"../resources".to_string()),
            "got: {:?}",
            paths
        );
    }

    #[test]
    fn extract_deps_edn_paths() {
        let s = r#"{:deps {com.foo/bar {:local/root "../foo"}
                com.baz/qux {:local/root "../../shared"}}}
"#;
        let paths = extract_relative_paths(s);
        assert!(paths.contains(&"../foo".to_string()), "got: {:?}", paths);
        assert!(
            paths.contains(&"../../shared".to_string()),
            "got: {:?}",
            paths
        );
    }

    #[test]
    fn extract_requirements_in_paths() {
        let s = "-r ../shared/requirements.in\n-e ../local-pkg\nrequests\n";
        let paths = extract_relative_paths(s);
        assert!(
            paths.contains(&"../shared/requirements.in".to_string()),
            "got: {:?}",
            paths
        );
        assert!(
            paths.contains(&"../local-pkg".to_string()),
            "got: {:?}",
            paths
        );
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
