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
    /// 2. Fast path: substitute a known non-canonical prefix for the canonical one (no syscalls).
    /// 3. Memoized canonicalize: first miss costs one syscall; subsequent lookups for the same path
    ///    are O(1).
    pub fn get(&self, dir: &Path) -> Option<&DirInfo> {
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
    pub fn descendants_of<'a>(
        &'a self,
        dir: &Path,
    ) -> impl Iterator<Item = (&'a Path, &'a DirInfo)> {
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
