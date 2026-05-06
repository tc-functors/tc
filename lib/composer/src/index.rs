//! Single-pass filesystem index shared by composer and differ.
//!
//! ## Why
//!
//! Before this module existed, `tc compose` and `tc diff` did several
//! overlapping filesystem walks of the same subtree. Real-world impact
//! on a moderately-sized topology (~150 functions across ~16 nested
//! sub-topologies): **~2.5 s wall-clock with ~16 s of system CPU**,
//! dominated by `stat`/`open`/`getdents`. The hot duplicated paths:
//!
//! - composer's `make_nodes` walked the repo to discover `topology.yml`
//!   files,
//! - composer's `discover_functions` per-topology listed subdirs and
//!   ran `path_exists` 8–10 times against each candidate to identify
//!   function dirs (`is_inferred_dir`),
//! - differ's `Analyzer` walked each function dir again to find
//!   symlinks and manifest files.
//!
//! ## What
//!
//! [`RepoIndex`] performs a single recursive walk from a chosen root
//! (the process pwd, see [`get`]) and records, for every directory it
//! visits, the names of regular files, the absolute paths of immediate
//! subdirectories, and the absolute paths of immediate symlinks.
//! Composer, differ, and any future consumer answer "does
//! `dir/foo.bar` exist?" / "what symlinks live under `fn_dir`?" /
//! "is this a topology dir?" with a hashmap lookup instead of a
//! syscall.
//!
//! ## Lifetime and invalidation
//!
//! Built lazily on first call to [`get`] and cached in a process-wide
//! `OnceLock`. `tc` is a one-shot CLI; the repo doesn't change between
//! the first read and process exit, so there is no invalidation
//! mechanism. Files written by the builder *after* compose (e.g.
//! `lambda.zip`) are not queried via the index — those callers go
//! straight to `Path::exists` / `kit::file_exists`.
//!
//! ## Footgun: pwd is captured at first call
//!
//! Because the index is a singleton seeded from `kit::pwd()` at first
//! call, anything that changes pwd between process start and the first
//! `index::get()` is permanently baked in. For one-shot CLI use this is
//! fine; library or test consumers that want a fresh index for a
//! different root must construct one with [`RepoIndex::build`] directly
//! and pass it around explicitly rather than going through [`get`].

use kit as u;
use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
    sync::{
        Mutex,
        OnceLock,
    },
};
use walkdir::WalkDir;

/// Directories whose contents are never relevant to topology / function
/// discovery and which can be huge in real monorepos (multi-GB `.git`
/// dirs alone take seconds to walk). Skipping them keeps the up-front
/// index build proportional to the topology size, not the repo size.
///
/// Selection criterion: the name must be either dot-prefixed (Unix
/// convention for hidden / cache / IDE state) or a universally-
/// recognised non-source name (`node_modules`, `__pycache__`).
/// Generic English words like `build`, `dist`, `target` are
/// **deliberately not** skipped — users could legitimately name a
/// topology or function dir any of those, and silently dropping them
/// would be a stealth regression vs the legacy `WalkDir`-based
/// discovery (which never pruned). If a future profiling pass shows
/// `target/` (Rust build output) actually matters for some user, this
/// list should become configurable rather than expanded.
///
/// **Side-effect on the differ:** because the differ's index path
/// iterates the index to find symlinks and manifest files under a
/// function dir (see `differ::deps::Analyzer::walk_and_extract`),
/// symlinks/manifests buried inside one of these subtrees are NOT
/// discovered. In practice these dirs are gitignored cache / IDE state
/// trees, so files inside them either don't show up in `git diff` at
/// all or aren't real source-code dependencies.
const SKIP_DIR_NAMES: &[&str] = &[
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    ".vendor",
    ".next",
    ".turbo",
    ".cache",
    ".tox",
    ".pytest_cache",
    ".mypy_cache",
    ".idea",
    ".vscode",
];

fn is_skipped_dir_name(name: &std::ffi::OsStr) -> bool {
    name.to_str()
        .map(|s| SKIP_DIR_NAMES.contains(&s))
        .unwrap_or(false)
}

/// Per-directory metadata captured by the single-pass walk.
///
/// **Always use [`DirInfo::has`] for membership checks** rather than
/// scanning [`Self::filenames`] directly: a `topology.yml` (or any
/// other file the composer / differ cares about) may be a symlink, and
/// raw filename iteration misses those — silently dropping work in a
/// way the legacy `WalkDir` + `Path::exists` codepath did not.
#[derive(Default, Clone, Debug)]
pub struct DirInfo {
    /// Regular-file basenames in this dir. Prefer [`Self::has`] over
    /// scanning this directly so symlinks are accounted for.
    pub filenames: Vec<String>,
    /// Absolute paths of immediate subdirectories.
    pub subdirs: Vec<PathBuf>,
    /// Absolute paths of immediate symlinks (file or dir).
    pub symlinks: Vec<PathBuf>,
}

impl DirInfo {
    /// True if `dir/<name>` resolves to an existing entity, matching
    /// `Path::exists()` / `kit::path_exists` semantics.
    ///
    /// The regular-file path is a plain string scan with no syscalls.
    /// On a symlink basename match the target is verified via
    /// `Path::exists()` so a dangling symlink returns `false` — without
    /// this, callers like `composer::aws::role::Role::new` would see
    /// the index report the role file as present, then panic in
    /// `kit::slurp` when `File::open` of the dangling target failed.
    /// Costs one extra syscall per symlink-name hit; the regular-file
    /// hot path is unchanged.
    pub fn has(&self, name: &str) -> bool {
        if self.filenames.iter().any(|f| f == name) {
            return true;
        }
        self.symlinks.iter().any(|p| {
            let basename_matches = p
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == name)
                .unwrap_or(false);
            basename_matches && p.exists()
        })
    }
}

/// In-memory snapshot of the filesystem rooted at [`Self::root`].
///
/// Keyed on canonical absolute paths. `non_canonical_root` records the
/// non-canonical form of the build root when it differs from canonical
/// (e.g. on macOS, `/tmp/repo` → `/private/tmp/repo`). Lookups try a
/// fast prefix swap before falling back to a `canonicalize()` syscall,
/// with the result of the slow path memoized in `alias`.
pub struct RepoIndex {
    root: PathBuf,
    dirs: HashMap<PathBuf, DirInfo>,
    non_canonical_root: Option<PathBuf>,
    alias: Mutex<HashMap<PathBuf, Option<PathBuf>>>,
}

impl RepoIndex {
    /// Walk `root` once and build the index. Symlinks are recorded but
    /// never followed, matching the (already-correct) behaviour of
    /// composer's `make_nodes` and differ's `Analyzer`.
    ///
    /// If `root` differs from its canonical form (e.g. `/tmp/repo` →
    /// `/private/tmp/repo` on macOS), the non-canonical prefix is
    /// recorded so per-query lookups can prefix-swap to canonical
    /// without a `canonicalize()` syscall.
    pub fn build(root: &Path) -> Self {
        let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let non_canonical_root = if root != canonical_root {
            Some(root.to_path_buf())
        } else {
            None
        };
        let mut dirs: HashMap<PathBuf, DirInfo> = HashMap::new();

        let mut iter = WalkDir::new(&canonical_root)
            .follow_links(false)
            .into_iter();
        loop {
            let entry = match iter.next() {
                Some(Ok(e)) => e,
                Some(Err(_)) => continue,
                None => break,
            };
            // Prune well-known noisy subtrees before recursing into them.
            // `.git`, `node_modules`, `target`, etc. can each contain
            // hundreds of thousands of entries that no composer or differ
            // code path cares about.
            if entry.file_type().is_dir()
                && entry.depth() > 0
                && is_skipped_dir_name(entry.file_name())
            {
                iter.skip_current_dir();
                continue;
            }
            let path = entry.path();
            let ft = entry.file_type();

            if ft.is_dir() {
                dirs.entry(path.to_path_buf()).or_default();
                if let Some(parent) = path.parent() {
                    if parent != path {
                        dirs.entry(parent.to_path_buf())
                            .or_default()
                            .subdirs
                            .push(path.to_path_buf());
                    }
                }
            } else if ft.is_symlink() {
                if let Some(parent) = path.parent() {
                    dirs.entry(parent.to_path_buf())
                        .or_default()
                        .symlinks
                        .push(path.to_path_buf());
                }
            } else if ft.is_file() {
                if let (Some(parent), Some(name)) =
                    (path.parent(), path.file_name().and_then(|n| n.to_str()))
                {
                    dirs.entry(parent.to_path_buf())
                        .or_default()
                        .filenames
                        .push(name.to_string());
                }
            }
        }

        for info in dirs.values_mut() {
            info.filenames.sort();
            info.subdirs.sort();
            info.symlinks.sort();
        }

        RepoIndex {
            root: canonical_root,
            dirs,
            non_canonical_root,
            alias: Mutex::new(HashMap::new()),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Lookup with canonicalization fallback. Accepts both absolute
    /// canonical paths and paths reachable from the canonical root via
    /// a symlink (e.g. `/tmp` → `/private/tmp` on macOS).
    ///
    /// Strategy:
    /// 1. Direct hashmap hit.
    /// 2. Fast path: substitute a known non-canonical prefix for the
    ///    canonical one (no syscalls).
    /// 3. Memoized canonicalize: first miss costs one syscall;
    ///    subsequent lookups for the same path are O(1).
    fn get(&self, dir: &Path) -> Option<&DirInfo> {
        if let Some(info) = self.dirs.get(dir) {
            return Some(info);
        }
        if let Some(alias_root) = &self.non_canonical_root {
            if let Ok(rel) = dir.strip_prefix(alias_root) {
                let candidate = self.root.join(rel);
                if let Some(info) = self.dirs.get(&candidate) {
                    return Some(info);
                }
            }
        }
        if let Ok(alias) = self.alias.lock() {
            if let Some(cached) = alias.get(dir) {
                return cached.as_ref().and_then(|p| self.dirs.get(p));
            }
        }
        let canonical = dir.canonicalize().ok();
        let result = canonical.as_ref().and_then(|c| self.dirs.get(c));
        if let Ok(mut alias) = self.alias.lock() {
            alias.insert(dir.to_path_buf(), canonical);
        }
        result
    }

    /// True if `dir/<name>` exists. Falls back to a real `Path::exists`
    /// syscall when `dir` is outside the indexed root.
    ///
    /// Note: this is *not* a drop-in for `kit::path_exists` in tests —
    /// `kit`'s test mock returns `path.contains("true")`, while we
    /// always go through real `Path::exists` on the fallback path.
    /// Composer's hot-path callers don't have unit tests that depend
    /// on the mock, so this asymmetry is benign in practice.
    pub fn path_exists(&self, dir: &str, name: &str) -> bool {
        match self.get(Path::new(dir)) {
            Some(info) => info.has(name),
            None => Path::new(&format!("{}/{}", dir, name)).exists(),
        }
    }

    /// True if `path` exists. Drop-in replacement for `kit::file_exists`
    /// for read-only compose-phase hot paths. Falls back to a syscall
    /// when the parent dir isn't in the index.
    pub fn file_exists(&self, path: &str) -> bool {
        let p = Path::new(path);
        let parent = match p.parent().and_then(|x| x.to_str()) {
            Some(parent_str) if !parent_str.is_empty() => parent_str,
            _ => return Path::new(path).exists(),
        };
        let name = match p.file_name().and_then(|x| x.to_str()) {
            Some(n) => n,
            None => return Path::new(path).exists(),
        };
        match self.get(Path::new(parent)) {
            Some(info) => info.has(name),
            None => Path::new(path).exists(),
        }
    }

    /// True iff `dir/topology.yml` exists.
    pub fn is_topology_dir(&self, dir: &str) -> bool {
        self.path_exists(dir, "topology.yml")
    }

    /// Mirrors composer/topology.rs::is_inferred_dir but answers from
    /// the index instead of doing 8 stat calls.
    pub fn is_inferred_dir(&self, dir: &str) -> bool {
        const MARKERS: &[&str] = &[
            "handler.rb",
            "handler.py",
            "main.go",
            "Cargo.toml",
            "handler.janet",
            "handler.clj",
            "handler.js",
            "main.janet",
        ];
        match self.get(Path::new(dir)) {
            Some(info) => MARKERS.iter().any(|m| info.has(m)),
            None => MARKERS.iter().any(|m| u::path_exists(dir, m)),
        }
    }

    /// Immediate subdirs of `dir` as strings, matching `kit::list_dirs`
    /// semantics: real subdirectories *and* symlinks that resolve to a
    /// directory are both returned. Empty if `dir` is unknown to the
    /// index.
    ///
    /// `info.subdirs` only holds real dirs because the build walk uses
    /// `follow_links(false)` and `entry.file_type().is_dir()` is
    /// `lstat`-based. `kit::list_dirs`, by contrast, calls
    /// `Path::is_dir()` which follows symlinks. Without the symlink
    /// branch below, a function dir that is itself a symlink to a real
    /// directory would be silently dropped from `function_dirs` —
    /// `tc compose` would behave identically to before this PR for all
    /// regular setups but would lose any symlinked function-dir entry
    /// at the topology root. The cost is one `metadata` syscall per
    /// symlink basename in `dir`, which is zero in the common case
    /// (topology dirs typically don't contain symlinks at the level
    /// queried by `function_dirs`).
    pub fn list_subdirs(&self, dir: &str) -> Vec<String> {
        match self.get(Path::new(dir)) {
            Some(info) => {
                let mut out: Vec<String> = info
                    .subdirs
                    .iter()
                    .filter_map(|p| p.to_str().map(|s| s.to_string()))
                    .collect();
                for sym in &info.symlinks {
                    if std::fs::metadata(sym).map(|m| m.is_dir()).unwrap_or(false) {
                        if let Some(s) = sym.to_str() {
                            out.push(s.to_string());
                        }
                    }
                }
                out
            }
            None => u::list_dirs(dir),
        }
    }

    /// All directories whose path starts with (or equals) `dir`. Used
    /// by the differ to scan a function's tree for symlinks and
    /// manifest files without re-walking, and by the composer to find
    /// nested topology.yml files in `make_nodes`.
    ///
    /// Iterates the entire `dirs` map and filters by prefix: O(N) in
    /// the size of the index. At current scale (a few hundred dirs per
    /// pwd-rooted topology) this is fine; if a future change makes the
    /// index span tens of thousands of dirs, consider a sorted Vec
    /// with binary-search range queries instead.
    pub fn descendants_of<'a>(&'a self, dir: &Path) -> impl Iterator<Item = (&'a Path, &'a DirInfo)> {
        let prefix = dir.to_path_buf();
        self.dirs
            .iter()
            .filter(move |(p, _)| p.starts_with(&prefix))
            .map(|(p, info)| (p.as_path(), info))
    }

    /// True iff every directory under `dir` is in the index. False for
    /// dirs outside the indexed root, in which case the differ should
    /// fall back to a real walk. Uses the same prefix-alias fast path
    /// as [`Self::get`] so a `covers()` check costs no syscalls when
    /// `dir` is reached via a known alias root.
    pub fn covers(&self, dir: &Path) -> bool {
        if dir.starts_with(&self.root) {
            return true;
        }
        if let Some(alias_root) = &self.non_canonical_root {
            if dir.starts_with(alias_root) {
                return true;
            }
        }
        match dir.canonicalize() {
            Ok(c) => c.starts_with(&self.root),
            Err(_) => false,
        }
    }
}

static CACHE: OnceLock<RepoIndex> = OnceLock::new();

/// Process-wide singleton index, rooted at `kit::pwd()`.
///
/// First call walks the pwd subtree; subsequent calls hit a cache.
/// Safe to call from any thread; the underlying `OnceLock` provides
/// one-time initialization.
///
/// We deliberately root at `pwd()` rather than `kit::root()` because
/// real monorepos can be enormous (multi-GB / hundreds-of-thousands of
/// files), of which a single topology is typically a tiny slice.
/// Single-threaded walks of the whole repo would blow several seconds
/// on the up-front build and serialize the otherwise-parallel composer
/// worker threads. The pwd subtree always covers the topology being
/// composed; symlinks that escape pwd are handled by the differ's
/// existing per-target `WalkDir` fallback (see
/// [`crate::index::RepoIndex::covers`]).
pub fn get() -> &'static RepoIndex {
    CACHE.get_or_init(|| {
        let pwd = u::pwd();
        tracing::debug!("building RepoIndex from {}", &pwd);
        let idx = RepoIndex::build(Path::new(&pwd));
        tracing::debug!("RepoIndex built: {} dirs", idx.dirs.len());
        idx
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn mk(root: &Path, rel: &str, contents: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, contents).unwrap();
    }

    #[test]
    fn finds_topology_dirs() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "topology.yml", "name: root");
        mk(root, "sub/topology.yml", "name: sub");
        mk(root, "sub/leaf/topology.yml", "name: leaf");
        mk(root, "other/handler.py", "");

        let idx = RepoIndex::build(root);
        assert!(idx.is_topology_dir(idx.root().to_str().unwrap()));
        assert!(idx.is_topology_dir(idx.root().join("sub").to_str().unwrap()));
        assert!(idx.is_topology_dir(idx.root().join("sub/leaf").to_str().unwrap()));
        assert!(!idx.is_topology_dir(idx.root().join("other").to_str().unwrap()));
    }

    #[test]
    fn detects_inferred_dirs_by_handler() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "py/handler.py", "");
        mk(root, "rb/handler.rb", "");
        mk(root, "rs/Cargo.toml", "");
        mk(root, "none/something_else.txt", "");

        let idx = RepoIndex::build(root);
        assert!(idx.is_inferred_dir(idx.root().join("py").to_str().unwrap()));
        assert!(idx.is_inferred_dir(idx.root().join("rb").to_str().unwrap()));
        assert!(idx.is_inferred_dir(idx.root().join("rs").to_str().unwrap()));
        assert!(!idx.is_inferred_dir(idx.root().join("none").to_str().unwrap()));
    }

    #[test]
    fn lists_subdirs() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "a/file.txt", "");
        mk(root, "b/file.txt", "");
        mk(root, "c/file.txt", "");

        let idx = RepoIndex::build(root);
        let mut subs = idx.list_subdirs(idx.root().to_str().unwrap());
        subs.sort();
        assert_eq!(subs.len(), 3);
        assert!(subs[0].ends_with("/a"));
        assert!(subs[1].ends_with("/b"));
        assert!(subs[2].ends_with("/c"));
    }

    /// Regression: `list_subdirs` previously returned only real dirs
    /// from `info.subdirs`, while the replaced `kit::list_dirs` used
    /// `p.is_dir()` which follows symlinks. A function dir that is
    /// itself a symlink to a real directory would have been silently
    /// dropped by `function_dirs` -> topology discovery. Verify that
    /// `list_subdirs` returns both real dirs and symlinks-to-dirs.
    #[test]
    fn list_subdirs_includes_symlinked_directories() {
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "real_fn/handler.py", "");
        mk(root, "shared/lib.py", "");
        // Function dir that is itself a symlink to another real dir.
        symlink(root.join("shared"), root.join("symlinked_fn")).unwrap();
        // A dangling dir-name symlink: must NOT appear because metadata
        // fails (matches kit::list_dirs's `p.is_dir()` semantics).
        symlink(root.join("does_not_exist"), root.join("dangling_dir")).unwrap();

        let idx = RepoIndex::build(root);
        let mut subs = idx.list_subdirs(idx.root().to_str().unwrap());
        subs.sort();
        let names: Vec<String> = subs
            .iter()
            .map(|s| {
                std::path::Path::new(s)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert!(names.contains(&"real_fn".to_string()), "missing real_fn: {:?}", names);
        assert!(names.contains(&"shared".to_string()), "missing shared: {:?}", names);
        assert!(
            names.contains(&"symlinked_fn".to_string()),
            "missing symlinked_fn (the regression): {:?}",
            names
        );
        assert!(
            !names.contains(&"dangling_dir".to_string()),
            "dangling symlink leaked into list_subdirs: {:?}",
            names
        );
    }

    #[test]
    fn covers_only_under_root() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "x/y.txt", "");
        let idx = RepoIndex::build(root);
        assert!(idx.covers(&idx.root().join("x")));
        // /tmp itself isn't under our indexed root.
        assert!(!idx.covers(Path::new("/usr/bin")));
    }

    #[test]
    fn dangling_symlink_does_not_report_as_existing() {
        // Regression: `DirInfo::has` previously returned true for any
        // symlink whose basename matched, even when the target was
        // missing. Callers like `Role::new` would then `slurp` the
        // dangling path and panic in `File::open`. Match
        // `Path::exists` semantics: dangling symlinks must report as
        // not-existing.
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("dir")).unwrap();
        symlink(
            root.join("nonexistent_target.json"),
            root.join("dir").join("vars.json"),
        )
        .unwrap();

        let idx = RepoIndex::build(root);
        let dir_str = idx.root().join("dir").to_str().unwrap().to_string();
        assert!(
            !idx.path_exists(&dir_str, "vars.json"),
            "dangling symlink must not report as existing"
        );
        assert!(
            !idx.file_exists(&format!("{}/vars.json", dir_str)),
            "dangling symlink must not report as existing via file_exists either"
        );
    }

    #[test]
    fn symlinked_topology_yml_is_visible_via_has() {
        // Regression test: callers iterating descendants_of and asking
        // `info.has("topology.yml")` should find topologies whose
        // `topology.yml` is a symlink, matching the legacy
        // `Path::exists` / `kit::path_exists` behaviour. An earlier
        // version of `nested_topology_dirs` and `list_topologies`
        // accidentally inlined `info.filenames.iter().any(...)`,
        // bypassing the symlink branch of `DirInfo::has` and silently
        // dropping symlinked topologies.
        use std::os::unix::fs::symlink;
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "real/topology.yml", "name: real");
        std::fs::create_dir_all(root.join("linked")).unwrap();
        symlink(
            root.join("real").join("topology.yml"),
            root.join("linked").join("topology.yml"),
        )
        .unwrap();

        let idx = RepoIndex::build(root);
        // Both dirs surface as topology dirs.
        assert!(idx.is_topology_dir(idx.root().join("real").to_str().unwrap()));
        assert!(idx.is_topology_dir(idx.root().join("linked").to_str().unwrap()));
        // ... and a descendants_of consumer using the supported
        // `info.has` API also picks both up.
        let canonical_root = idx.root().to_path_buf();
        let mut found: Vec<&Path> = idx
            .descendants_of(&canonical_root)
            .filter(|(p, info)| *p != canonical_root.as_path() && info.has("topology.yml"))
            .map(|(p, _)| p)
            .collect();
        found.sort();
        assert_eq!(found.len(), 2, "expected both real and linked topology dirs: {:?}", found);
    }

    #[test]
    fn skip_dir_names_are_pruned() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "src/handler.py", "");
        // Each of these should be invisible to the index — neither the
        // dir itself nor any descendant should appear.
        mk(root, ".git/objects/pack/pack-deadbeef", "");
        mk(root, "node_modules/foo/index.js", "");
        mk(root, "__pycache__/x.pyc", "");
        mk(root, ".venv/lib/python/site-packages/foo.py", "");

        let idx = RepoIndex::build(root);
        assert!(idx.path_exists(idx.root().join("src").to_str().unwrap(), "handler.py"));
        for skipped in [".git", "node_modules", "__pycache__", ".venv"] {
            assert!(
                idx.get(&idx.root().join(skipped)).is_none(),
                "{} should not be indexed",
                skipped
            );
        }
    }

    /// Regression: `build`, `dist`, and `target` are common English
    /// words that users might legitimately use as topology- or
    /// function-dir names. An earlier revision of `SKIP_DIR_NAMES`
    /// included them, and `descendants_of` / `list_subdirs` consequently
    /// dropped them — silently breaking nested-topology discovery in
    /// `nested_topology_dirs` and function-dir discovery in
    /// `function_dirs`. Verify they're now indexed and surfaced through
    /// the same APIs the composer uses.
    #[test]
    fn generic_named_dirs_are_not_pruned() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "build/topology.yml", "name: build");
        mk(root, "dist/topology.yml", "name: dist");
        mk(root, "target/handler.py", "");

        let idx = RepoIndex::build(root);

        // descendants_of drives nested_topology_dirs in composer.
        let canonical_root = idx.root().to_path_buf();
        let mut topology_descendants: Vec<&Path> = idx
            .descendants_of(&canonical_root)
            .filter(|(p, info)| *p != canonical_root.as_path() && info.has("topology.yml"))
            .map(|(p, _)| p)
            .collect();
        topology_descendants.sort();
        assert_eq!(topology_descendants.len(), 2);
        assert!(topology_descendants[0].ends_with("build"));
        assert!(topology_descendants[1].ends_with("dist"));

        // list_subdirs drives function_dirs in composer.
        let mut subs = idx.list_subdirs(idx.root().to_str().unwrap());
        subs.sort();
        assert_eq!(subs.len(), 3, "expected build, dist, target — got {:?}", subs);
        assert!(subs.iter().any(|s| s.ends_with("/build")));
        assert!(subs.iter().any(|s| s.ends_with("/dist")));
        assert!(subs.iter().any(|s| s.ends_with("/target")));

        // is_inferred_dir on the target dir works via the index.
        assert!(idx.is_inferred_dir(idx.root().join("target").to_str().unwrap()));
    }

    #[test]
    fn non_canonical_root_alias_avoids_canonicalize() {
        // Build the index using a path that includes a `..` segment,
        // which differs from its canonical form. Lookups using either
        // form should hit without falling through to the disk-touching
        // alias cache.
        let tmp = TempDir::new().unwrap();
        let canonical_root = tmp.path().canonicalize().unwrap();
        mk(&canonical_root, "fn/handler.py", "");

        // `<root>/fn/../fn` resolves to `<root>/fn`. We pass
        // `<root>/fn/..` to build, which canonicalizes to `<root>`.
        let non_canonical = canonical_root.join("fn").join("..");
        let idx = RepoIndex::build(&non_canonical);

        assert_eq!(idx.root(), canonical_root.as_path());
        // Direct (canonical) lookup hits.
        assert!(idx.path_exists(canonical_root.join("fn").to_str().unwrap(), "handler.py"));
        // Non-canonical-prefix lookup also hits via prefix swap.
        assert!(idx.path_exists(non_canonical.join("fn").to_str().unwrap(), "handler.py"));
    }

    #[test]
    fn path_exists_falls_back_outside_index() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        mk(root, "inside/x.txt", "");
        let idx = RepoIndex::build(root);

        // A real file outside the indexed root: must report true via
        // the syscall fallback (not just say "unknown == false").
        let outside = TempDir::new().unwrap();
        mk(outside.path(), "marker.txt", "");
        assert!(idx.path_exists(outside.path().to_str().unwrap(), "marker.txt"));
        assert!(!idx.path_exists(outside.path().to_str().unwrap(), "no_such_file"));
    }
}
