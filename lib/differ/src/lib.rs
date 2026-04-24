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

/// Collect every `(owning-topology-dir, function-name, Function)` tuple
/// reachable from `topology`, recursively descending into nested nodes.
/// The topology dir is kept alongside each function so the transducer
/// policy can still apply per-topology.
fn collect_functions<'a>(
    topology: &'a Topology,
    out: &mut Vec<(&'a Topology, &'a String, &'a Function)>,
) {
    for (name, f) in &topology.functions {
        out.push((topology, name, f));
    }
    for (_, node) in &topology.nodes {
        collect_functions(node, out);
    }
}

/// Collect every subtopology (self + nested), so transducer policy can
/// apply to each owning topology independently.
fn collect_topologies<'a>(topology: &'a Topology, out: &mut Vec<&'a Topology>) {
    out.push(topology);
    for (_, node) in &topology.nodes {
        collect_topologies(node, out);
    }
}

/// Filter the (recursively-collected) functions of `topology` down to
/// those whose code-dep closure intersects the git diff between `from`
/// and `to`.
///
/// First-deploy semantics: if `from` is empty, all functions (including
/// transducers) are returned.
pub fn diff_fns(
    topology: &Topology,
    from: &str,
    to: &str,
) -> HashMap<String, Function> {
    let all_topologies = {
        let mut v = Vec::new();
        collect_topologies(topology, &mut v);
        v
    };
    let all_functions = {
        let mut v = Vec::new();
        collect_functions(topology, &mut v);
        v
    };
    tracing::debug!(
        "differ scanning {} topology(ies), {} function(s)",
        all_topologies.len(),
        all_functions.len()
    );

    let first_deploy = from.is_empty();
    if first_deploy {
        tracing::debug!(
            "no prior version for {} — marking {} functions + transducers changed",
            &topology.namespace,
            all_functions.len()
        );
        let mut out: HashMap<String, Function> = all_functions
            .iter()
            .map(|(_, name, f)| ((*name).clone(), (*f).clone()))
            .collect();
        for t in &all_topologies {
            if let Some(tx) = &t.transducer {
                out.insert(tx.name.clone(), tx.function.clone());
            }
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

    let mut changed: HashMap<String, Function> = HashMap::new();
    for (_owning, name, f) in &all_functions {
        let closure = analyzer.closure(&PathBuf::from(&f.dir));
        if diff.intersects(&closure) {
            changed.insert((*name).clone(), (*f).clone());
        }
    }

    // Transducer Policy B: per owning topology, include the transducer if
    // any diff path is under the topology's own dir.
    for t in &all_topologies {
        if let Some(tx) = &t.transducer {
            let topo_dir = match PathBuf::from(&t.dir).canonicalize() {
                Ok(p) => p,
                Err(_) => continue,
            };
            if diff.any_under(&topo_dir) {
                changed.insert(tx.name.clone(), tx.function.clone());
            }
        }
    }

    changed
}

/// CLI helper: print the functions that changed between two tags, plus a
/// changelog from the `tagger` crate.
pub fn diff(topology: &Topology, from: &str, to: &str) {
    let fns = diff_fns(topology, from, to);

    println!("Modified functions:");
    let mut names: Vec<&String> = fns.keys().collect();
    names.sort();
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
