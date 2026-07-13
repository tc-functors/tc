//! Per-function code-dependency closure computation with shared caching.
//!
//! The core data structure is [`Analyzer`]: a cache of directory →
//! "outgoing refs" (symlink targets + manifest path-dep resolutions). Every
//! unique directory in the repo is walked at most once per `Analyzer`
//! lifetime, regardless of how many functions include it in their closure.
//! For a topology where 100 functions all symlink to the same `shared_lib`,
//! `shared_lib` is walked exactly once.
//!
//! A function's closure is a BFS over the directed graph whose edges are
//! these cached refs, starting from `f.dir`. The returned [`Closure`] is a
//! small set of "walked-root" directories (any descendant file matches) and
//! a small set of extra file paths (exact match for individual
//! symlinked-file targets).

use crate::manifest;
use std::{
    cell::RefCell,
    collections::{
        BTreeSet,
        HashMap,
        VecDeque,
    },
    fs,
    path::{
        Path,
        PathBuf,
    },
};
use walkdir::WalkDir;

/// Upper bound on symlink-chain depth. Defensive against pathological
/// chains or disk corruption.
const MAX_SYMLINK_DEPTH: usize = 16;

/// Outgoing edges from a single directory: other dirs / files it depends
/// on. Does NOT include descendant subdirs walked during the directory's
/// own walk — those are covered implicitly by the fact that this dir is a
/// closure root.
#[derive(Debug, Default, Clone)]
struct Refs {
    dirs: Vec<PathBuf>,
    files: Vec<PathBuf>,
}

/// Final result of a per-function closure computation.
#[derive(Debug, Default, Clone)]
pub struct Closure {
    /// Walked-root canonical dirs. Any descendant file under any of these
    /// is considered part of the closure.
    pub roots: BTreeSet<PathBuf>,
    /// Extra absolute canonical file paths (e.g. a symlink target that
    /// is a single file).
    pub files: BTreeSet<PathBuf>,
}

impl Closure {
    /// Returns true iff `abs_path` is under any root in the closure or
    /// exactly equal to one of the extra files.
    pub fn contains(&self, abs_path: &Path) -> bool {
        if self.files.contains(abs_path) {
            return true;
        }
        for r in &self.roots {
            if abs_path.starts_with(r) {
                return true;
            }
        }
        false
    }
}

/// Cross-function cache. One instance per diff invocation.
pub struct Analyzer {
    repo_root: PathBuf,
    refs: RefCell<HashMap<PathBuf, Refs>>,
}

impl Analyzer {
    /// Construct with `repo_root`. The root is canonicalized once up-front.
    /// Returns `None` if `repo_root` cannot be canonicalized.
    pub fn new(repo_root: &Path) -> Option<Self> {
        let canonical = repo_root.canonicalize().ok()?;
        Some(Self {
            repo_root: canonical,
            refs: RefCell::new(HashMap::new()),
        })
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Compute the closure of `fn_dir`, then widen it with explicit
    /// per-function aux file paths (role JSONs, vars JSONs, etc. — files
    /// outside the source-code closure whose contents flow into the
    /// deployed configuration).
    ///
    /// Each aux path is canonicalized; if it doesn't resolve on disk
    /// (deleted between tags, or a conventional path that was never
    /// created), we keep the logical path so a `DiffSet` entry produced
    /// by the same `repo_root.join(rel)` fallback still byte-matches and
    /// the function is correctly flagged. Aux paths that resolve outside
    /// `repo_root` are dropped — a defense-in-depth measure since the
    /// composer shouldn't produce them.
    ///
    /// **Why a wrapper instead of a parameter on `closure()`.**
    /// `closure()` populates the per-dir `Refs` cache keyed by directory.
    /// Aux paths are per-function, not per-dir, so threading them through
    /// `closure()` would either lose caching or require a separate cache
    /// layer. Composing `closure(fn_dir) + per-function aux insert`
    /// keeps the dir-level cache intact.
    pub fn closure_with_aux(&self, fn_dir: &Path, aux: &[PathBuf]) -> Closure {
        let mut c = self.closure(fn_dir);
        for p in aux {
            match p.canonicalize() {
                Ok(canon) => {
                    if canon.starts_with(&self.repo_root) {
                        c.files.insert(canon);
                    }
                }
                Err(_) => {
                    c.files.insert(p.clone());
                }
            }
        }
        c
    }

    /// Compute the closure of `fn_dir`. If the dir doesn't exist or is
    /// outside the repo, returns an empty closure.
    pub fn closure(&self, fn_dir: &Path) -> Closure {
        let seed = match self.canonicalize_within(fn_dir) {
            Some(s) => s,
            None => return Closure::default(),
        };

        let mut roots: BTreeSet<PathBuf> = BTreeSet::new();
        let mut files: BTreeSet<PathBuf> = BTreeSet::new();
        let mut queue: VecDeque<PathBuf> = VecDeque::new();
        queue.push_back(seed);

        while let Some(d) = queue.pop_front() {
            if !roots.insert(d.clone()) {
                continue;
            }
            let refs = self.refs_for(&d);
            for nd in &refs.dirs {
                if !roots.contains(nd) {
                    queue.push_back(nd.clone());
                }
            }
            for f in &refs.files {
                files.insert(f.clone());
            }
        }

        Closure { roots, files }
    }

    /// Canonicalize `p` and confirm it's inside `repo_root`. Returns None
    /// if it doesn't exist or escapes the repo.
    fn canonicalize_within(&self, p: &Path) -> Option<PathBuf> {
        let c = p.canonicalize().ok()?;
        if c.starts_with(&self.repo_root) {
            Some(c)
        } else {
            None
        }
    }

    /// Get the cached refs for `dir`, populating the cache if needed.
    fn refs_for(&self, dir: &Path) -> Refs {
        if let Some(r) = self.refs.borrow().get(dir) {
            return r.clone();
        }
        let r = self.walk_and_extract(dir);
        self.refs.borrow_mut().insert(dir.to_path_buf(), r.clone());
        r
    }

    fn walk_and_extract(&self, dir: &Path) -> Refs {
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut files: Vec<PathBuf> = Vec::new();
        let mut manifests: Vec<PathBuf> = Vec::new();

        let idx = composer::index::get();
        if idx.covers(dir) {
            // Fast path: composer already walked the topology subtree;
            // filter its per-dir snapshot to the subtree under `dir`
            // instead of re-walking. `dir` is always canonical here:
            // `closure()` canonicalizes the seed via
            // `canonicalize_within`, and subsequent queue entries come
            // from `resolve_symlink_chain` / `resolve_manifest_refs`,
            // both of which return canonical paths.
            //
            // Note: the index prunes well-known noisy subtrees (`.git`,
            // `node_modules`, `target`, etc. — see
            // [`composer::index::SKIP_DIR_NAMES`]). Symlinks and
            // manifests inside those subtrees are NOT visible here; the
            // legacy `WalkDir` fallback below walks everything. In
            // practice these dirs are gitignored or build-artifact
            // trees that wouldn't drive code-deploy decisions anyway.
            for (dir_path, info) in idx.descendants_of(dir) {
                for sym in &info.symlinks {
                    if let Some(target) = resolve_symlink_chain(sym, &self.repo_root) {
                        match fs::metadata(&target) {
                            Ok(meta) if meta.is_dir() => dirs.push(target),
                            Ok(meta) if meta.is_file() => files.push(target),
                            Ok(_) => {}
                            Err(e) => tracing::warn!(
                                "symlink {} resolved to unreadable target {}: {}",
                                sym.display(),
                                target.display(),
                                e
                            ),
                        }
                    }
                }
                for fname in &info.filenames {
                    if manifest::is_manifest(fname) {
                        manifests.push(dir_path.join(fname));
                    }
                }
            }
        } else {
            for entry in WalkDir::new(dir)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                let ft = entry.file_type();

                if ft.is_symlink() {
                    if let Some(target) = resolve_symlink_chain(path, &self.repo_root) {
                        match fs::metadata(&target) {
                            Ok(meta) if meta.is_dir() => dirs.push(target),
                            Ok(meta) if meta.is_file() => files.push(target),
                            Ok(_) => {}
                            Err(e) => tracing::warn!(
                                "symlink {} resolved to unreadable target {}: {}",
                                path.display(),
                                target.display(),
                                e
                            ),
                        }
                    }
                } else if ft.is_file() {
                    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                        if manifest::is_manifest(name) {
                            manifests.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        for m in &manifests {
            for target in resolve_manifest_refs(m, &self.repo_root) {
                match fs::metadata(&target) {
                    Ok(meta) if meta.is_dir() => dirs.push(target),
                    Ok(meta) if meta.is_file() => files.push(target),
                    _ => {}
                }
            }
        }

        dirs.sort();
        dirs.dedup();
        files.sort();
        files.dedup();
        Refs { dirs, files }
    }
}

/// Convenience wrapper: build an Analyzer and compute a single closure.
/// Use the long-lived `Analyzer` directly when computing closures for
/// many functions — cross-function caching depends on it.
pub fn compute_closure(fn_dir: &Path, repo_root: &Path) -> Closure {
    match Analyzer::new(repo_root) {
        Some(a) => a.closure(fn_dir),
        None => Closure::default(),
    }
}

/// Resolve `symlink_path` through any chain, returning the canonical target
/// if it lives inside `repo_root` (already canonicalized). Warns and
/// returns None on chain-too-deep, broken link, or off-repo target.
fn resolve_symlink_chain(symlink_path: &Path, repo_root: &Path) -> Option<PathBuf> {
    let mut current = symlink_path.to_path_buf();
    for _ in 0..MAX_SYMLINK_DEPTH {
        let meta = match fs::symlink_metadata(&current) {
            Ok(m) => m,
            Err(_) => return None,
        };
        if !meta.file_type().is_symlink() {
            let canonical = current.canonicalize().ok()?;
            if canonical.starts_with(repo_root) {
                return Some(canonical);
            } else {
                tracing::debug!(
                    "symlink {} -> {} escapes repo {} — ignoring",
                    symlink_path.display(),
                    canonical.display(),
                    repo_root.display()
                );
                return None;
            }
        }
        let target = fs::read_link(&current).ok()?;
        current = if target.is_absolute() {
            target
        } else {
            current.parent()?.join(target)
        };
    }
    tracing::warn!(
        "symlink chain exceeds depth {} at {}",
        MAX_SYMLINK_DEPTH,
        symlink_path.display()
    );
    None
}

/// Scan a manifest file for relative-path tokens and resolve each against
/// the manifest's dir. Returns the canonical targets that exist on disk
/// and live inside `repo_root`. Warns on tokens that fail to resolve.
fn resolve_manifest_refs(manifest_path: &Path, repo_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let contents = match fs::read_to_string(manifest_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("failed to read manifest {}: {}", manifest_path.display(), e);
            return out;
        }
    };

    let parent = match manifest_path.parent() {
        Some(p) => p,
        None => return out,
    };

    for token in manifest::extract_relative_paths(&contents) {
        let joined = parent.join(&token);
        match joined.canonicalize() {
            Ok(canonical) => {
                if canonical.starts_with(repo_root) {
                    out.push(canonical);
                }
            }
            Err(_) => {
                tracing::warn!(
                    "manifest {} references {} which does not resolve — skipping",
                    manifest_path.display(),
                    joined.display()
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::symlink,
    };
    use tempfile::TempDir;

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

    #[test]
    fn closure_is_just_fn_dir_when_isolated() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(
            root,
            "functions/foo/handler.py",
            "def handler(e, c): pass\n",
        );
        let c = compute_closure(&root.join("functions/foo"), root);
        assert_eq!(c.roots.len(), 1);
        assert!(c.files.is_empty());
    }

    #[test]
    fn closure_follows_pyproject_path_dep() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "functions/foo/handler.py", "");
        mkfile(
            root,
            "functions/foo/pyproject.toml",
            r#"shared = {path = "../../shared"}"#,
        );
        mkfile(root, "shared/core.py", "x = 1");
        let c = compute_closure(&root.join("functions/foo"), root);
        let shared = root.join("shared").canonicalize().unwrap();
        assert!(c.roots.contains(&shared));
    }

    #[test]
    fn closure_transitive_across_path_deps() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "functions/foo/handler.py", "");
        mkfile(
            root,
            "functions/foo/pyproject.toml",
            r#"a = {path = "../../shared-a"}"#,
        );
        mkfile(
            root,
            "shared-a/pyproject.toml",
            r#"b = {path = "../shared-b"}"#,
        );
        mkfile(root, "shared-a/src/a.py", "");
        mkfile(root, "shared-b/src/b.py", "");
        let c = compute_closure(&root.join("functions/foo"), root);
        assert!(
            c.roots
                .contains(&root.join("shared-a").canonicalize().unwrap())
        );
        assert!(
            c.roots
                .contains(&root.join("shared-b").canonicalize().unwrap())
        );
    }

    #[test]
    fn closure_handles_cycles() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "functions/foo/handler.py", "");
        mkfile(
            root,
            "functions/foo/pyproject.toml",
            r#"a = {path = "../../shared-a"}"#,
        );
        mkfile(
            root,
            "shared-a/pyproject.toml",
            r#"b = {path = "../shared-b"}"#,
        );
        mkfile(
            root,
            "shared-b/pyproject.toml",
            r#"a = {path = "../shared-a"}"#,
        );
        let c = compute_closure(&root.join("functions/foo"), root);
        assert!(
            c.roots
                .contains(&root.join("shared-a").canonicalize().unwrap())
        );
        assert!(
            c.roots
                .contains(&root.join("shared-b").canonicalize().unwrap())
        );
    }

    #[test]
    fn closure_follows_symlink_to_dir() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkdir(root, "functions/foo");
        mkdir(root, "shared");
        mkfile(root, "shared/util.py", "x = 1");
        symlink(root.join("shared"), root.join("functions/foo/shared")).unwrap();
        let c = compute_closure(&root.join("functions/foo"), root);
        assert!(
            c.roots
                .contains(&root.join("shared").canonicalize().unwrap())
        );
    }

    #[test]
    fn closure_follows_deep_nested_symlink() {
        // Mimics a common real-repo pattern: each function has a handler
        // subdir which contains a symlink to ../../../shared_lib.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "shared_lib/utils/a.rb", "");
        mkdir(root, "functions/foo/handler");
        symlink(
            root.join("shared_lib"),
            root.join("functions/foo/handler/shared_lib"),
        )
        .unwrap();
        let c = compute_closure(&root.join("functions/foo"), root);
        assert!(
            c.roots
                .contains(&root.join("shared_lib").canonicalize().unwrap())
        );
    }

    #[test]
    fn closure_ignores_symlink_escaping_repo() {
        let outer = TempDir::new().unwrap();
        let repo = TempDir::new().unwrap();
        let root = repo.path();
        mkdir(root, "functions/foo");
        mkdir(outer.path(), "external");
        fs::write(outer.path().join("external/x.txt"), "x").unwrap();
        symlink(
            outer.path().join("external"),
            root.join("functions/foo/external"),
        )
        .unwrap();
        let c = compute_closure(&root.join("functions/foo"), root);
        let outer_canonical = outer.path().join("external").canonicalize().unwrap();
        assert!(!c.roots.contains(&outer_canonical));
    }

    #[test]
    fn closure_empty_for_nonexistent_fn_dir() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        let c = compute_closure(&root.join("/tmp/tc/synthetic"), root);
        assert!(c.roots.is_empty() && c.files.is_empty());
    }

    #[test]
    fn closure_warns_on_missing_path_dep() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "functions/foo/handler.py", "");
        mkfile(
            root,
            "functions/foo/pyproject.toml",
            r#"missing = {path = "../../does-not-exist"}"#,
        );
        let c = compute_closure(&root.join("functions/foo"), root);
        let missing = root.join("does-not-exist");
        assert!(!c.roots.contains(&missing));
    }

    #[test]
    fn contains_works_for_file_under_root() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "functions/foo/handler.py", "");
        let c = compute_closure(&root.join("functions/foo"), root);
        let abs = root
            .canonicalize()
            .unwrap()
            .join("functions/foo/handler.py");
        assert!(c.contains(&abs));
        assert!(!c.contains(&root.canonicalize().unwrap().join("other/bar.py")));
    }

    #[test]
    fn contains_works_for_symlinked_file_target() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkdir(root, "functions/foo");
        mkfile(root, "shared/util.py", "");
        symlink(
            root.join("shared/util.py"),
            root.join("functions/foo/util.py"),
        )
        .unwrap();
        let c = compute_closure(&root.join("functions/foo"), root);
        let abs = root.canonicalize().unwrap().join("shared/util.py");
        assert!(c.contains(&abs));
    }

    #[test]
    fn analyzer_caches_shared_dir_across_functions() {
        // Two functions symlinking to the same shared dir should walk the
        // shared dir exactly once.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "shared_lib/a.rb", "");
        mkfile(root, "shared_lib/b.rb", "");
        mkdir(root, "functions/foo");
        mkdir(root, "functions/bar");
        symlink(
            root.join("shared_lib"),
            root.join("functions/foo/shared_lib"),
        )
        .unwrap();
        symlink(
            root.join("shared_lib"),
            root.join("functions/bar/shared_lib"),
        )
        .unwrap();

        let analyzer = Analyzer::new(root).unwrap();
        let foo = analyzer.closure(&root.join("functions/foo"));
        let bar = analyzer.closure(&root.join("functions/bar"));

        let shared = root.join("shared_lib").canonicalize().unwrap();
        assert!(foo.roots.contains(&shared));
        assert!(bar.roots.contains(&shared));

        // Three unique dirs were reffed (foo, bar, shared_lib) — the
        // analyzer's cache should show exactly those entries.
        let refs = analyzer.refs.borrow();
        assert_eq!(refs.len(), 3, "cache contents: {:?}", refs.keys());
    }

    #[test]
    fn aux_picks_up_role_file_outside_fn_dir() {
        // A function's role JSON lives in the infrastructure tree, not
        // under the function's source dir. closure_with_aux must include
        // the canonical role path so role-only edits flag the function.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(
            root,
            "topologies/foo/myfn/handler.py",
            "def handler(): pass\n",
        );
        mkfile(root, "infrastructure/tc/foo/roles/myfn.json", "{}");

        let analyzer = Analyzer::new(root).unwrap();
        let aux = vec![root.join("infrastructure/tc/foo/roles/myfn.json")];
        let c = analyzer.closure_with_aux(&root.join("topologies/foo/myfn"), &aux);

        let canonical = root
            .join("infrastructure/tc/foo/roles/myfn.json")
            .canonicalize()
            .unwrap();
        assert!(c.contains(&canonical));
    }

    #[test]
    fn aux_handles_deleted_file_path() {
        // Conventional aux paths are always emitted by the composer,
        // even when the file doesn't exist on disk (e.g. deleted between
        // tags). closure_with_aux must keep the logical path so a
        // DiffSet entry for the deletion (also a logical join) matches.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "topologies/foo/myfn/handler.py", "");

        let analyzer = Analyzer::new(root).unwrap();
        let logical = root.join("infrastructure/tc/foo/roles/myfn.json");
        assert!(!logical.exists(), "test setup: file must NOT exist");

        let c = analyzer.closure_with_aux(&root.join("topologies/foo/myfn"), &[logical.clone()]);

        assert!(
            c.files.contains(&logical),
            "logical (non-canonical) path must be retained for deletion case; \
             closure files = {:?}",
            c.files
        );
    }

    #[test]
    fn aux_drops_paths_outside_repo() {
        // A real-on-disk aux path that resolves outside the repo must be
        // silently dropped. Defense in depth — composer shouldn't produce
        // these, but if it does we don't want false matches against an
        // unrelated tree.
        let outer = TempDir::new().unwrap();
        let repo = TempDir::new().unwrap();
        let root = repo.path();
        mkfile(root, "topologies/foo/myfn/handler.py", "");
        mkfile(outer.path(), "stray/role.json", "{}");

        let analyzer = Analyzer::new(root).unwrap();
        let outside = outer.path().join("stray/role.json");
        let c = analyzer.closure_with_aux(&root.join("topologies/foo/myfn"), &[outside.clone()]);

        let canonical = outside.canonicalize().unwrap();
        assert!(!c.contains(&canonical), "outside-repo aux path leaked in");
    }

    #[test]
    fn aux_canonicalizes_symlinks() {
        // If the aux path itself is a symlink, the canonical target is
        // what ends up in the closure — and it's also what the diff side
        // produces from a `repo_root.join(rel).canonicalize()`. Match.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "topologies/foo/myfn/handler.py", "");
        mkfile(root, "real/role.json", "{}");
        symlink(root.join("real/role.json"), root.join("link.json")).unwrap();

        let analyzer = Analyzer::new(root).unwrap();
        let c =
            analyzer.closure_with_aux(&root.join("topologies/foo/myfn"), &[root.join("link.json")]);

        let canonical_real = root.join("real/role.json").canonicalize().unwrap();
        assert!(c.contains(&canonical_real));
    }

    #[test]
    fn aux_does_not_double_count_paths_in_source_closure() {
        // A path that's already covered by a closure root (because it
        // sits under f.dir) and is also explicitly listed as aux must
        // not produce inconsistent results. `Closure::contains` should
        // still return true for that path; no duplicate-flagging hazard.
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mkfile(root, "topologies/foo/myfn/handler.py", "");
        mkfile(root, "topologies/foo/myfn/role.json", "{}");

        let analyzer = Analyzer::new(root).unwrap();
        let aux_path = root.join("topologies/foo/myfn/role.json");
        let c = analyzer.closure_with_aux(&root.join("topologies/foo/myfn"), &[aux_path.clone()]);

        let canonical = aux_path.canonicalize().unwrap();
        assert!(c.contains(&canonical));
    }
}
