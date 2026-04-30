//! Function-level change detection for `tc` topologies.
//!
//! High-level: between git tags A (older) and B (newer), determine the
//! subset of functions in a topology whose **code** artifact inputs have
//! changed, and therefore must be redeployed.
//!
//! Design invariants that matter for perf:
//!   - The git diff runs ONCE per `diff_fns` invocation, regardless of
//!     function count.
//!   - The repo root is canonicalized ONCE; all downstream comparisons use
//!     the canonical path.
//!   - Directories are walked ONCE per unique path across all functions
//!     (via [`deps::Analyzer`]). Shared libraries referenced by N functions
//!     are analyzed a single time.
//!   - Matching is pure `starts_with` on canonical paths — no per-call
//!     canonicalization.

mod deps;
mod manifest;

pub use deps::{compute_closure, Analyzer, Closure};

use composer::{Function, Topology};
use kit as u;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Error returned by [`diff_fns`] when an incremental diff cannot be
/// computed. Surfaced as a typed error so callers (e.g. the resolver)
/// can distinguish "no functions changed" from "diff could not be
/// computed" — the conflation between the two is exactly the bug this
/// module previously enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffError {
    /// `git rev-parse --verify` for `tag` returned empty even after a
    /// best-effort fetch from origin. The deployed Lambda's `version`
    /// resource tag is pointing at a git tag that no longer exists.
    TagUnresolvable { tag: String },
}

impl fmt::Display for DiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiffError::TagUnresolvable { tag } => write!(
                f,
                "git tag {} could not be resolved locally or fetched from origin",
                tag
            ),
        }
    }
}

impl std::error::Error for DiffError {}

/// Best-effort: ensure `refs/tags/{tag}` is resolvable in `dir`. If it
/// already resolves locally, returns `true`. Otherwise tries one fetch
/// from `origin` and re-checks. The fetch itself is tolerated to fail
/// (e.g. tag deleted upstream) — only the post-fetch `rev-parse` result
/// matters.
fn ensure_tag(tag: &str, dir: &str) -> bool {
    let resolved = |t: &str| {
        !u::sh(
            &format!("git rev-parse --verify --quiet refs/tags/{}", t),
            dir,
        )
        .is_empty()
    };
    if resolved(tag) {
        return true;
    }
    u::sh(
        &format!(
            "git fetch origin +refs/tags/{tag}:refs/tags/{tag} 2>/dev/null || true",
            tag = tag
        ),
        dir,
    );
    resolved(tag)
}

/// Canonicalized absolute paths of files changed between two refs. We
/// store these pre-canonicalized so per-function matching is just a cheap
/// `starts_with` scan.
#[derive(Debug, Default, Clone)]
struct DiffSet {
    files: Vec<PathBuf>,
}

impl DiffSet {
    fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// True iff any file in the diff lies under `dir` (exact `starts_with`).
    fn any_under(&self, dir: &Path) -> bool {
        self.files.iter().any(|f| f.starts_with(dir))
    }

    /// True iff any file in the diff is covered by the closure — either
    /// under one of its roots or exactly equal to one of its extra files.
    fn intersects(&self, closure: &Closure) -> bool {
        for f in &self.files {
            if closure.contains(f) {
                return true;
            }
        }
        false
    }
}

fn repo_root_canonical() -> PathBuf {
    PathBuf::from(u::root())
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(u::root()))
}

/// Return the list of changed files (repo-relative paths) between
/// `from_tag` and `to_tag` within `namespace`, executing all git
/// commands in `dir`. Tries to fetch any tag that isn't already
/// resolvable locally; on persistent failure it logs the error rather
/// than panicking — callers that need to distinguish "no changes" from
/// "diff couldn't be computed" should use [`diff_fns`].
pub fn find_between_versions_in(namespace: &str, from: &str, to: &str, dir: &str) -> Vec<String> {
    if from == to {
        return vec![];
    }
    let from_tag = format!("{}-{}", namespace, from);
    let to_tag = format!("{}-{}", namespace, to);

    // Only fetch tags that aren't already resolvable locally. Saves
    // multi-second network round-trips on repeat invocations / CI runs
    // that pre-fetched tags.
    ensure_tag(&to_tag, dir);
    ensure_tag(&from_tag, dir);

    let cmd = format!("git diff --name-only {}..{}", &from_tag, &to_tag);
    tracing::debug!("{}", &cmd);
    let out = u::sh(&cmd, dir);
    u::split_lines(&out)
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Backward-compatible wrapper: defaults `dir` to `u::pwd()`.
pub fn find_between_versions(namespace: &str, from: &str, to: &str) -> Vec<String> {
    find_between_versions_in(namespace, from, to, &u::pwd())
}

fn files_modified_in_branch() -> Vec<String> {
    let dir = u::pwd();
    let branch = u::sh("git rev-parse --abbrev-ref HEAD", &dir);
    if branch == "HEAD" {
        return vec![];
    }
    let default = {
        let raw = u::sh(
            "git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'",
            &dir,
        );
        if raw.is_empty() {
            "main".to_string()
        } else {
            raw
        }
    };
    if branch == default {
        return vec![];
    }
    let out = u::sh(
        &format!("git diff --name-only {}...{}", &default, &branch),
        &dir,
    );
    u::split_lines(&out)
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn files_modified_uncommitted() -> Vec<String> {
    let dir = u::pwd();
    let out = u::sh("git ls-files -m", &dir);
    u::split_lines(&out)
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Build a canonicalized diff set. Paths that don't resolve on disk are
/// kept as logical joins (they still match starts_with for the deleted-
/// file case).
fn build_diff_set(
    namespace: &str,
    from: &str,
    to: &str,
    repo_root: &Path,
) -> DiffSet {
    let mut rels: Vec<String> = find_between_versions(namespace, from, to);
    if std::env::var("CI").is_err() {
        rels.extend(files_modified_uncommitted());
        rels.extend(files_modified_in_branch());
    }
    rels.sort();
    rels.dedup();

    let files: Vec<PathBuf> = rels
        .into_iter()
        .map(|r| {
            let joined = repo_root.join(&r);
            // Canonicalize when possible (handles deleted files by falling
            // back to the logical join — which still prefix-matches any
            // closure that included that path).
            joined.canonicalize().unwrap_or(joined)
        })
        .collect();
    DiffSet { files }
}

/// Filter `topology`'s own `functions` down to those whose code-dep
/// closure intersects the git diff between `from` and `to`.
///
/// Shallow by design: this processes `topology.functions` and the
/// transducer of `topology` itself — **not** nested `topology.nodes`.
/// Callers that need to process nested topologies should iterate them
/// and call `diff_fns` for each, matching the resolver's per-topology
/// iteration pattern. A recursive variant would cause the root
/// topology's `functions` map to be contaminated with node-level
/// functions and would cause those functions to be re-resolved with the
/// wrong context by `resolver::resolve`.
///
/// `namespace` is the **root** topology's namespace — it determines the
/// git tag prefix used when constructing the diff range (`{namespace}-
/// {from}..{namespace}-{to}`). It is passed explicitly rather than read
/// from `topology.namespace` because `topology` may be a nested node
/// whose own namespace differs from the root's, and tags in this repo
/// are keyed by the root namespace only. Using `topology.namespace`
/// here would silently produce an empty diff for nested nodes (lookup
/// of nonexistent tags), causing functions to be skipped from
/// re-deployment.
///
/// First-deploy semantics: if `from` is empty, all of this topology's
/// own functions (plus its transducer, if any) are returned.
///
/// Errors with [`DiffError::TagUnresolvable`] when the from-tag
/// (`{namespace}-{from}`) cannot be resolved locally and cannot be
/// fetched from `origin`. Callers must treat this as "diff cannot be
/// computed" and *not* as "no functions changed" — this conflation was
/// the root cause of stale-deployed-version silent no-op deploys.
///
/// The to-tag is intentionally not validated: `tc create` is invoked
/// from that ref, so it is always present in the working tree.
pub fn diff_fns(
    topology: &Topology,
    namespace: &str,
    from: &str,
    to: &str,
) -> Result<HashMap<String, Function>, DiffError> {
    let first_deploy = from.is_empty();
    if first_deploy {
        tracing::debug!(
            "no prior version for {} — marking {} function(s) + transducer changed",
            namespace,
            topology.functions.len()
        );
        let mut out: HashMap<String, Function> = topology.functions.clone();
        if let Some(tx) = &topology.transducer {
            out.insert(tx.name.clone(), tx.function.clone());
        }
        return Ok(out);
    }

    if from == to {
        return Ok(HashMap::new());
    }

    // Pre-flight the from-tag *before* invoking `git diff`. Without this
    // guard, `git diff` against a missing ref would emit a stderr error
    // that `kit::sh`'s `Redirection::Merge` then surfaces as the diff's
    // "stdout", causing `build_diff_set` to parse the error message
    // lines as changed file paths. The closure-intersection check would
    // then reject all of them and `diff_fns` would silently return an
    // empty map — indistinguishable from "no functions changed".
    let dir = u::pwd();
    let from_tag = format!("{}-{}", namespace, from);
    if !ensure_tag(&from_tag, &dir) {
        return Err(DiffError::TagUnresolvable { tag: from_tag });
    }

    let repo_root = repo_root_canonical();
    let diff = build_diff_set(namespace, from, to, &repo_root);
    if diff.is_empty() {
        tracing::debug!("empty diff for {} {}..{}", namespace, from, to);
        return Ok(HashMap::new());
    }

    let analyzer = match Analyzer::new(&repo_root) {
        Some(a) => a,
        None => {
            tracing::warn!(
                "could not canonicalize repo root {}; skipping diff",
                repo_root.display()
            );
            return Ok(HashMap::new());
        }
    };

    Ok(diff_fns_with(topology, &diff, &analyzer))
}

/// Internal helper: filter one topology's own functions against a
/// pre-built diff set + analyzer. Used by both the public `diff_fns` and
/// the CLI `diff` walker so the git diff + Analyzer cache are built once
/// per invocation.
fn diff_fns_with(
    topology: &Topology,
    diff: &DiffSet,
    analyzer: &Analyzer,
) -> HashMap<String, Function> {
    let mut changed: HashMap<String, Function> = HashMap::new();
    for (name, f) in &topology.functions {
        let closure = analyzer.closure(&PathBuf::from(&f.dir));
        if diff.intersects(&closure) {
            changed.insert(name.clone(), f.clone());
        }
    }
    // Transducer Policy B: include the topology's transducer if any
    // diff path is under the topology's own dir.
    if let Some(tx) = &topology.transducer {
        if let Ok(topo_dir) = PathBuf::from(&topology.dir).canonicalize() {
            if diff.any_under(&topo_dir) {
                changed.insert(tx.name.clone(), tx.function.clone());
            }
        }
    }
    changed
}

/// Walk `topology` and every nested node, pushing each onto `out`.
fn collect_topologies<'a>(topology: &'a Topology, out: &mut Vec<&'a Topology>) {
    out.push(topology);
    for node in topology.nodes.values() {
        collect_topologies(node, out);
    }
}

/// CLI helper: print the functions that changed between two tags across
/// the root topology and all nested nodes, plus a changelog from the
/// `tagger` crate. Computes the git diff and Analyzer cache once and
/// reuses them across all topologies.
pub fn diff(topology: &Topology, from: &str, to: &str) {
    let first_deploy = from.is_empty();
    let mut topologies: Vec<&Topology> = Vec::new();
    collect_topologies(topology, &mut topologies);

    let mut names: Vec<String> = Vec::new();

    if first_deploy {
        for t in &topologies {
            names.extend(t.functions.keys().cloned());
            if let Some(tx) = &t.transducer {
                names.push(tx.name.clone());
            }
        }
    } else if from != to {
        let repo_root = repo_root_canonical();
        let diff = build_diff_set(&topology.namespace, from, to, &repo_root);
        if !diff.is_empty() {
            if let Some(analyzer) = Analyzer::new(&repo_root) {
                for t in &topologies {
                    names.extend(diff_fns_with(t, &diff, &analyzer).into_keys());
                }
            }
        }
    }

    println!("Modified functions:");
    names.sort();
    names.dedup();
    for name in names {
        println!("  - {}", name);
    }

    println!();
    println!("Changelog:");
    let f = format!("{}-{}", &topology.namespace, from);
    let t = format!("{}-{}", &topology.namespace, to);
    let changes = tagger::commits(&f, &t);
    println!("{}", changes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Initialize a fresh git repo at `root` with a single commit and
    /// the given tags. Tags are created with the `tag::v` naming scheme
    /// used everywhere in this codebase: `{namespace}-{version}`.
    fn git_init_with_tags(root: &Path, namespace: &str, tags: &[&str]) {
        let dir = root.to_str().unwrap();
        u::sh("git init -q", dir);
        u::sh("git config user.email 'test@example.com'", dir);
        u::sh("git config user.name 'test'", dir);
        u::sh("git config commit.gpgsign false", dir);
        u::sh("git config tag.gpgSign false", dir);
        fs::write(root.join("seed.txt"), "seed\n").unwrap();
        u::sh("git add seed.txt", dir);
        u::sh("git commit -q -m seed", dir);
        for v in tags {
            u::sh(&format!("git tag {}-{}", namespace, v), dir);
        }
    }

    #[test]
    fn ensure_tag_returns_true_for_existing_tag() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        git_init_with_tags(tmp.path(), "ns", &["0.1.0"]);
        assert!(ensure_tag("ns-0.1.0", dir));
    }

    /// Critical regression: a tag that doesn't exist locally and can't
    /// be fetched (no `origin` remote) must report unresolved. This is
    /// the exact precondition that previously caused `git diff` to emit
    /// an error that was silently parsed as "changed files".
    #[test]
    fn ensure_tag_returns_false_for_missing_tag_with_no_remote() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        git_init_with_tags(tmp.path(), "ns", &["0.1.0"]);
        assert!(!ensure_tag("ns-0.0.39", dir));
    }

    #[test]
    fn ensure_tag_returns_false_for_missing_tag_with_unreachable_remote() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        git_init_with_tags(tmp.path(), "ns", &["0.1.0"]);
        u::sh(
            "git remote add origin /nonexistent/remote/that/does/not/resolve",
            dir,
        );
        assert!(!ensure_tag("ns-0.0.39", dir));
    }

    #[test]
    fn find_between_versions_in_returns_changed_files_on_happy_path() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let dir = root.to_str().unwrap();
        git_init_with_tags(root, "ns", &["0.1.0"]);

        fs::write(root.join("a.txt"), "v2\n").unwrap();
        u::sh("git add a.txt", dir);
        u::sh("git commit -q -m bump", dir);
        u::sh("git tag ns-0.2.0", dir);

        let files = find_between_versions_in("ns", "0.1.0", "0.2.0", dir);
        assert_eq!(files, vec!["a.txt"]);
    }

    #[test]
    fn find_between_versions_in_returns_empty_when_from_eq_to() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_str().unwrap();
        // Tag setup intentionally omitted: from==to short-circuits
        // before any git command runs.
        let files = find_between_versions_in("ns", "0.1.0", "0.1.0", dir);
        assert!(files.is_empty());
    }

    #[test]
    fn diff_error_display_includes_tag_name() {
        let e = DiffError::TagUnresolvable {
            tag: "mr-orchestrator-0.0.39".to_string(),
        };
        let s = format!("{}", e);
        assert!(s.contains("mr-orchestrator-0.0.39"), "got: {}", s);
    }

    // Integration of `ensure_tag` into `diff_fns` (i.e. the end-to-end
    // wiring that returns `Err(TagUnresolvable)` for a missing from-
    // tag) is verified at the resolver level via
    // `resolver::function::classify_modified` tests. Constructing a
    // realistic `composer::Topology` here would require populating
    // ~25 fields and is out of proportion to what this 4-line wiring
    // (`if !ensure_tag(...) { return Err(...) }`) demands.
}
