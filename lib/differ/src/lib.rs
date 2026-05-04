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

fn files_modified_in_branch_in(dir: &str) -> Vec<String> {
    let branch = u::sh("git rev-parse --abbrev-ref HEAD", dir);
    if branch == "HEAD" {
        return vec![];
    }
    let default = {
        let raw = u::sh(
            "git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@'",
            dir,
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
        dir,
    );
    u::split_lines(&out)
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Tracked-and-modified files across the whole repo. Run from the repo
/// root so changes outside the current dir — most notably role JSONs
/// under `infrastructure/tc/...` — are visible regardless of where `tc`
/// was invoked.
fn files_modified_uncommitted_in(dir: &str) -> Vec<String> {
    let out = u::sh("git ls-files -m", dir);
    u::split_lines(&out)
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Untracked files (gitignore-respecting). Included in the local diff
/// set so brand-new files — e.g. a freshly-added role JSON under
/// `infrastructure/tc/.../roles/{fn}.json` — are visible to the closure
/// intersection check before they're committed. `--exclude-standard`
/// keeps gitignored build artifacts (`lambda.zip`, `target/`, etc.) out.
/// Run from the repo root so paths outside cwd are also reported.
fn files_untracked_in(dir: &str) -> Vec<String> {
    let out = u::sh("git ls-files --others --exclude-standard", dir);
    u::split_lines(&out)
        .iter()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Build a canonicalized diff set. Paths that don't resolve on disk are
/// kept as logical joins (they still match starts_with for the deleted-
/// file case).
///
/// `include_local` controls whether the working-tree's modified,
/// branch-vs-default, and untracked files are folded into the diff in
/// addition to the strict `from..to` git diff.
///
/// - **Deploy time** (`diff_fns`) → `true`. The deploy is going from
///   the snapshotter-recorded version to the current local code on
///   disk; uncommitted local edits are part of "current local code" and
///   the deploy must see them.
/// - **CLI inspection** (`diff`, called from `tc diff --between A..B`)
///   → `false`. Both endpoints are explicit committed tags, so the
///   working-tree state is irrelevant. Including it produces noisy
///   false positives in checkouts with lots of untracked files (test
///   payloads, scratch docs, etc.) that happen to live under a
///   function's source dir.
fn build_diff_set(
    namespace: &str,
    from: &str,
    to: &str,
    repo_root: &Path,
    include_local: bool,
) -> DiffSet {
    let dir = repo_root.to_str().unwrap_or_else(|| {
        // Should never fire — `repo_root` is canonicalized, valid UTF-8
        // on every platform we run on. Belt-and-suspenders fallback.
        ""
    });
    let mut rels: Vec<String> = find_between_versions_in(namespace, from, to, dir);
    if include_local && std::env::var("CI").is_err() {
        rels.extend(files_modified_uncommitted_in(dir));
        rels.extend(files_modified_in_branch_in(dir));
        rels.extend(files_untracked_in(dir));
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
    // Deploy-time path: include local working-tree state because the
    // deploy is going from the recorded version to current local code
    // and uncommitted edits are part of what's about to be shipped.
    let diff = build_diff_set(namespace, from, to, &repo_root, true);
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
        // Aux files (role JSONs, vars JSONs, inherited parent role)
        // live outside f.dir and the source-code closure walker can't
        // reach them. The composer enumerates them on `Runtime` so the
        // differ can widen the per-function closure here. A change to
        // any of these files marks the function dirty even when no
        // source code changed; the existing deploy path
        // (`lambda::create_or_update`) then writes the full config —
        // role attachment, env block, memory, timeout — using the
        // freshly composed Runtime values.
        let aux: Vec<PathBuf> = f
            .runtime
            .aux_files
            .iter()
            .map(PathBuf::from)
            .collect();
        let closure = analyzer.closure_with_aux(&PathBuf::from(&f.dir), &aux);
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
        // CLI inspection path: strict tag-to-tag diff. Don't fold in
        // the local working tree — the user is asking "what changed
        // between these two committed versions" and untracked test
        // payloads / scratch files in the checkout would otherwise
        // produce false positives.
        let diff = build_diff_set(&topology.namespace, from, to, &repo_root, false);
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

    // ---- Helpers ----

    /// Initialize a fresh git repo at `root` with a single commit and
    /// the given tags. Tags are created with the `tag::v` naming scheme
    /// used everywhere in this codebase: `{namespace}-{version}`. Used
    /// by the `ensure_tag` / `find_between_versions_in` tests where a
    /// lightweight repo with no working-tree dirt is enough.
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

    fn mkfile(root: &Path, rel: &str, contents: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, contents).unwrap();
    }

    fn mkdir(root: &Path, rel: &str) {
        fs::create_dir_all(root.join(rel)).unwrap();
    }

    /// Build a minimal `Topology` containing a single function with a
    /// customizable `dir` and `aux_files`. Constructed via `serde_json`
    /// rather than by hand because the fixture surface is otherwise
    /// enormous (Runtime alone has ~20 fields, plus nested
    /// Role/Policy/Trust/etc.). Tests that depend on a particular field
    /// should set it explicitly through the JSON template.
    fn fixture_topology(fn_name: &str, fn_dir: &str, aux_files: &[String]) -> Topology {
        let aux_json = serde_json::to_string(aux_files).unwrap();
        let json = format!(
            r#"{{
                "namespace": "test",
                "env": "dev",
                "fqn": "test",
                "concurrent": false,
                "kind": "Function",
                "infra": "",
                "dir": "/tmp/tc-test-topology",
                "sandbox": "test",
                "hyphenated_names": false,
                "version": "0.0.0",
                "nodes": {{}},
                "events": {{}},
                "routes": {{}},
                "functions": {{
                    "{fn_name}": {{
                        "name": "{fn_name}",
                        "actual_name": "{fn_name}",
                        "namespace": "test",
                        "dir": "{fn_dir}",
                        "description": null,
                        "fqn": "test_{fn_name}",
                        "arn": "",
                        "layer_name": null,
                        "version": "",
                        "runtime": {{
                            "lang": "Python310",
                            "provider": "Lambda",
                            "handler": "handler.handler",
                            "package_type": "zip",
                            "uri": "",
                            "layers": [],
                            "tags": {{}},
                            "environment": {{}},
                            "memory_size": null,
                            "cpu": null,
                            "timeout": null,
                            "snapstart": false,
                            "provisioned_concurrency": null,
                            "reserved_concurrency": null,
                            "enable_fs": false,
                            "network": null,
                            "fs": null,
                            "role": {{
                                "name": "tc-base-function-test",
                                "kind": "Base",
                                "path": "",
                                "trust": {{"Version": "2012-10-17", "Statement": []}},
                                "arn": "",
                                "policy_name": "tc-base-function-test",
                                "policy": {{"Version": "2012-10-17", "Statement": []}},
                                "policy_arn": ""
                            }},
                            "infra_spec": {{}},
                            "cluster": "",
                            "aux_files": {aux_json}
                        }},
                        "build": {{
                            "dir": "{fn_dir}",
                            "kind": "Code",
                            "pre": [],
                            "post": [],
                            "version": null,
                            "command": "",
                            "pack": "",
                            "shared_context": false,
                            "skip_dev_deps": false,
                            "environment": {{}}
                        }},
                        "test": {{}},
                        "targets": []
                    }}
                }},
                "all_functions": {{}},
                "mutations": {{}},
                "schedules": {{}},
                "queues": {{}},
                "channels": {{}},
                "pools": {{}},
                "pages": {{}},
                "tags": {{}},
                "flow": null,
                "config": {{
                    "aws": {{
                        "lambda": {{
                            "layers_profile": null,
                            "extensions_profile": null,
                            "role": null
                        }},
                        "fmt": {{
                            "fqn": "default"
                        }}
                    }},
                    "ci": {{
                        "build": {{
                            "kind": "code",
                            "version_image": false
                        }}
                    }},
                    "builder": {{
                        "kind": "Local",
                        "cluster": null
                    }}
                }},
                "roles": {{}},
                "base_roles": {{}},
                "tests": {{}},
                "transducer": null,
                "sequences": {{}}
            }}"#
        );
        serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("topology fixture failed to deserialize: {e}; json = {json}"))
    }

    /// End-to-end through `diff_fns_with`: when a role file outside the
    /// function's source dir is in the diff, the function is flagged.
    #[test]
    fn diff_fns_flags_function_when_only_role_file_changed() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "topologies/foo/myfn/handler.py", "");
        mkfile(root, "infrastructure/tc/foo/roles/myfn.json", "{\"old\": true}");

        let fn_dir = root.join("topologies/foo/myfn").to_str().unwrap().to_string();
        let role_path = root
            .join("infrastructure/tc/foo/roles/myfn.json")
            .to_str()
            .unwrap()
            .to_string();
        let topology = fixture_topology("myfn", &fn_dir, &[role_path.clone()]);

        let analyzer = Analyzer::new(root).unwrap();

        // Simulate `build_diff_set` for a modified file: canonicalize
        // the changed path. (Modification case — file exists.)
        let canonical = PathBuf::from(&role_path).canonicalize().unwrap();
        let diff = DiffSet { files: vec![canonical] };

        let changed = diff_fns_with(&topology, &diff, &analyzer);
        assert!(
            changed.contains_key("myfn"),
            "function must be flagged when only its role file changed; \
             changed = {:?}",
            changed.keys().collect::<Vec<_>>()
        );
    }

    /// End-to-end through `diff_fns_with` for the deletion case: a
    /// vars file that existed at tag A but not at tag B. The differ
    /// records the deletion as a logical join of repo_root + rel; the
    /// composer emits the same logical path in `aux_files` because the
    /// conventional path is always emitted regardless of on-disk
    /// existence. Both must match for the function to be flagged.
    #[test]
    fn diff_fns_flags_function_when_only_vars_file_deleted() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "topologies/foo/myfn/handler.py", "");
        mkdir(root, "infrastructure/tc/foo");
        // NOTE: vars/myfn.json deliberately does NOT exist on disk —
        // this is the deletion-at-tag-B case.

        let fn_dir = root.join("topologies/foo/myfn").to_str().unwrap().to_string();
        // The composer would emit the conventional path even though the
        // file is gone — mirror that here.
        let canonical_root = root.canonicalize().unwrap();
        let logical_vars_path = canonical_root
            .join("infrastructure/tc/foo/vars/myfn.json")
            .to_str()
            .unwrap()
            .to_string();
        let topology =
            fixture_topology("myfn", &fn_dir, &[logical_vars_path.clone()]);

        let analyzer = Analyzer::new(root).unwrap();

        // Mirror `build_diff_set`'s deletion fallback: canonicalize
        // fails, so we keep the logical join.
        let rel = "infrastructure/tc/foo/vars/myfn.json";
        let joined = canonical_root.join(rel);
        let diff_path = joined.canonicalize().unwrap_or(joined);
        let diff = DiffSet { files: vec![diff_path] };

        let changed = diff_fns_with(&topology, &diff, &analyzer);
        assert!(
            changed.contains_key("myfn"),
            "function must be flagged when its vars file was deleted; \
             changed = {:?}",
            changed.keys().collect::<Vec<_>>()
        );
    }

    /// Sanity check: an empty diff (nothing changed) does not flag the
    /// function even though it has aux_files declared.
    #[test]
    fn diff_fns_does_not_flag_when_diff_is_empty() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "topologies/foo/myfn/handler.py", "");
        mkfile(root, "infrastructure/tc/foo/roles/myfn.json", "{}");

        let fn_dir = root.join("topologies/foo/myfn").to_str().unwrap().to_string();
        let role_path = root
            .join("infrastructure/tc/foo/roles/myfn.json")
            .to_str()
            .unwrap()
            .to_string();
        let topology = fixture_topology("myfn", &fn_dir, &[role_path]);

        let analyzer = Analyzer::new(root).unwrap();
        let diff = DiffSet { files: vec![] };

        let changed = diff_fns_with(&topology, &diff, &analyzer);
        assert!(
            changed.is_empty(),
            "no diff means no function should be flagged; changed = {:?}",
            changed.keys().collect::<Vec<_>>()
        );
    }

    // -------------------------------------------------------------------
    // `include_local` gating regression tests
    //
    // These tests exercise a regression that shipped briefly: the CLI
    // helper `tc diff --between A..B` was pulling in the local working
    // tree (modified, branch-vs-default, untracked) on top of the
    // strict tag-to-tag git diff. That's correct for the deploy path
    // (`diff_fns`, where local edits are part of what's about to ship),
    // but wrong for inspection — checkouts with lots of untracked test
    // payloads / scratch docs under a function's source dir produced
    // false positives unrelated to anything that actually changed
    // between the two tags.
    //
    // The fix threaded an `include_local: bool` through `build_diff_set`
    // and the helpers it calls. These tests pin that contract: they
    // build a real git repo in a TempDir and verify the gate's effect
    // end-to-end.
    // -------------------------------------------------------------------

    /// Run `git` in `dir` via raw `Command` (NOT `u::sh`) so we don't
    /// depend on the parent test process's PWD or `u::root()`'s
    /// OnceLock cache. Returns the trimmed stdout. Panics on non-zero
    /// exit so misconfigured fixtures fail loudly.
    fn git(dir: &Path, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            // Provide author identity inline — the test environment
            // may not have a global git config.
            .env("GIT_AUTHOR_NAME", "tc-test")
            .env("GIT_AUTHOR_EMAIL", "tc-test@localhost")
            .env("GIT_COMMITTER_NAME", "tc-test")
            .env("GIT_COMMITTER_EMAIL", "tc-test@localhost")
            .output()
            .unwrap_or_else(|e| panic!("git {:?} in {:?} failed to spawn: {e}", args, dir));
        if !out.status.success() {
            panic!(
                "git {:?} in {:?} failed: stdout={} stderr={}",
                args,
                dir,
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Synthetic git history with three tags:
    ///
    /// - `0.0.1` (A): commits `handler.py` + `roles/myfn.json`.
    /// - `0.0.2` (B): modifies `roles/myfn.json`.
    /// - `0.0.3` (C): adds `otherfn/handler.py`.
    ///
    /// After the last tag, the working tree is dirtied with:
    /// - an untracked `topologies/foo/myfn/scratch.json` (the kind of
    ///   stray test payload that the bug was flagging).
    /// - an uncommitted modification to `otherfn/handler.py`.
    ///
    /// The strict tag-to-tag diff between any pair of these tags
    /// should never surface the post-tag dirt.
    fn synthetic_repo(namespace: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        git(dir, &["init", "--quiet", "--initial-branch=main"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        // Disable any global hooks that might inject content.
        git(dir, &["config", "core.hooksPath", "/dev/null"]);

        // Tag A.
        mkfile(dir, "topologies/foo/myfn/handler.py", "v1\n");
        mkfile(dir, "infrastructure/tc/foo/roles/myfn.json", "{\"v\": 1}");
        git(dir, &["add", "."]);
        git(dir, &["commit", "--quiet", "-m", "v1"]);
        git(dir, &["tag", &format!("{}-0.0.1", namespace)]);

        // Tag B: change one tracked file.
        fs::write(
            dir.join("infrastructure/tc/foo/roles/myfn.json"),
            "{\"v\": 2}",
        )
        .unwrap();
        git(dir, &["add", "."]);
        git(dir, &["commit", "--quiet", "-m", "v2"]);
        git(dir, &["tag", &format!("{}-0.0.2", namespace)]);

        // Tag C: add a new function dir + handler.
        mkfile(dir, "topologies/foo/otherfn/handler.py", "v1\n");
        git(dir, &["add", "."]);
        git(dir, &["commit", "--quiet", "-m", "add otherfn"]);
        git(dir, &["tag", &format!("{}-0.0.3", namespace)]);

        // ---- Working-tree noise (deliberately AFTER all tags + adds) ----
        // Stray untracked file under a function's source dir. This is
        // the kind of file (test payload, scratch doc, etc.) that the
        // bug was incorrectly flagging in `tc diff --between A..B`.
        mkfile(dir, "topologies/foo/myfn/scratch.json", "untracked");
        // Uncommitted modification to a tracked file. Also must be
        // invisible to a strict tag-to-tag inspection.
        fs::write(dir.join("topologies/foo/otherfn/handler.py"), "v1-edited\n").unwrap();

        tmp
    }

    /// **Regression test.** When `include_local=false`, the diff set
    /// must contain only what `git diff A..B` reports — never the
    /// working tree's modifications, branch-vs-default diff, or
    /// untracked files.
    ///
    /// Reproduces the bug behind `tc diff --between 0.0.333..0.0.334`
    /// reporting six functions when only two had genuinely changed
    /// (the four extras were dirs containing untracked test payloads).
    #[test]
    fn build_diff_set_excludes_local_state_when_include_local_is_false() {
        let tmp = synthetic_repo("ns");
        let dir = tmp.path();

        // Sanity: the working tree contains the noise we expect.
        assert!(
            dir.join("topologies/foo/myfn/scratch.json").exists(),
            "fixture: untracked file must exist"
        );

        let diff = build_diff_set("ns", "0.0.2", "0.0.3", dir, false);
        let paths: Vec<String> = diff
            .files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        // 0.0.2 -> 0.0.3 introduced topologies/foo/otherfn/handler.py.
        assert!(
            paths.iter().any(|p| p.ends_with("topologies/foo/otherfn/handler.py")),
            "tag-to-tag diff must surface the committed change; got {:?}",
            paths
        );
        // The untracked scratch file must NOT appear.
        assert!(
            !paths.iter().any(|p| p.ends_with("topologies/foo/myfn/scratch.json")),
            "untracked working-tree file leaked into a strict tag-to-tag \
             diff (this is the regression we're guarding against); got {:?}",
            paths
        );
        // The diff between two committed tags should be exactly one
        // path: the committed addition. No spillover from the
        // working-tree noise the fixture stages after the tag.
        assert_eq!(
            paths.len(),
            1,
            "strict tag-to-tag diff should contain exactly the committed \
             change; got {:?}",
            paths
        );
    }

    /// Counterpart: when `include_local=true` (the deploy-time path),
    /// the same untracked file IS picked up. This verifies that the
    /// gating is the only behavioral difference, so the fix doesn't
    /// silently regress the deploy-time semantics.
    #[test]
    fn build_diff_set_includes_local_state_when_include_local_is_true() {
        // Skip when running under CI=1 — `build_diff_set` short-circuits
        // local-state collection there for the same reason real CI
        // runs do (avoid noisy uncommitted state on shared runners).
        // Setting/unsetting CI globally inside a single test is racy
        // because env mutation isn't thread-safe across cargo's parallel
        // test runner; punt cleanly instead.
        if std::env::var("CI").is_ok() {
            eprintln!("skipping include_local=true test under CI=1");
            return;
        }

        let tmp = synthetic_repo("ns");
        let dir = tmp.path();

        let diff = build_diff_set("ns", "0.0.2", "0.0.3", dir, true);
        let paths: Vec<String> = diff
            .files
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        assert!(
            paths.iter().any(|p| p.ends_with("topologies/foo/myfn/scratch.json")),
            "include_local=true must surface untracked working-tree \
             files (deploy-time semantics); got {:?}",
            paths
        );
    }

    /// Empty `from..to` range: no committed changes, and we still want
    /// the gate to do the right thing for the inspection path. With
    /// `include_local=false` and a clean (no-op) tag pair, the diff
    /// set must be empty regardless of working-tree state.
    #[test]
    fn build_diff_set_is_empty_for_same_tag_when_include_local_is_false() {
        let tmp = synthetic_repo("ns");
        let dir = tmp.path();

        let diff = build_diff_set("ns", "0.0.2", "0.0.2", dir, false);
        assert!(
            diff.files.is_empty(),
            "from == to with include_local=false must yield empty diff; \
             got {:?}",
            diff.files
        );
    }

    // ---- ensure_tag / find_between_versions_in (from upstream PR #67) ----

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
