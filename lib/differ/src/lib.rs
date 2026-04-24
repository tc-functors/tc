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
use std::path::{Path, PathBuf};

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
/// `from_tag` and `to_tag` within `namespace`.
pub fn find_between_versions(namespace: &str, from: &str, to: &str) -> Vec<String> {
    if from == to {
        return vec![];
    }
    let dir = u::pwd();
    let from_tag = format!("{}-{}", namespace, from);
    let to_tag = format!("{}-{}", namespace, to);

    // Only fetch tags that aren't already resolvable locally. Saves
    // multi-second network round-trips on repeat invocations / CI runs
    // that pre-fetched tags.
    if u::sh(
        &format!("git rev-parse --verify --quiet refs/tags/{}", &to_tag),
        &dir,
    )
    .is_empty()
    {
        u::sh(
            &format!(
                "git fetch origin +refs/tags/{tag}:refs/tags/{tag} 2>/dev/null || true",
                tag = &to_tag
            ),
            &dir,
        );
    }
    if u::sh(
        &format!("git rev-parse --verify --quiet refs/tags/{}", &from_tag),
        &dir,
    )
    .is_empty()
    {
        u::sh(
            &format!(
                "git fetch origin +refs/tags/{tag}:refs/tags/{tag} 2>/dev/null || true",
                tag = &from_tag
            ),
            &dir,
        );
    }

    let cmd = format!("git diff --name-only {}..{}", &from_tag, &to_tag);
    tracing::debug!("{}", &cmd);
    let out = u::sh(&cmd, &dir);
    u::split_lines(&out)
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
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
/// First-deploy semantics: if `from` is empty, all of this topology's
/// own functions (plus its transducer, if any) are returned.
pub fn diff_fns(
    topology: &Topology,
    from: &str,
    to: &str,
) -> HashMap<String, Function> {
    let first_deploy = from.is_empty();
    if first_deploy {
        tracing::debug!(
            "no prior version for {} — marking {} function(s) + transducer changed",
            &topology.namespace,
            topology.functions.len()
        );
        let mut out: HashMap<String, Function> = topology.functions.clone();
        if let Some(tx) = &topology.transducer {
            out.insert(tx.name.clone(), tx.function.clone());
        }
        return out;
    }

    if from == to {
        return HashMap::new();
    }

    let repo_root = repo_root_canonical();
    let diff = build_diff_set(&topology.namespace, from, to, &repo_root);
    if diff.is_empty() {
        tracing::debug!("empty diff for {} {}..{}", &topology.namespace, from, to);
        return HashMap::new();
    }

    let analyzer = match Analyzer::new(&repo_root) {
        Some(a) => a,
        None => {
            tracing::warn!(
                "could not canonicalize repo root {}; skipping diff",
                repo_root.display()
            );
            return HashMap::new();
        }
    };

    diff_fns_with(topology, &diff, &analyzer)
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
