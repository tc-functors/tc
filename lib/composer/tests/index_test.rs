use std::fs;
use tempfile::TempDir;
use std::path::Path;
use composer::index::*;

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
    assert!(
        names.contains(&"real_fn".to_string()),
        "missing real_fn: {:?}",
        names
    );
    assert!(
        names.contains(&"shared".to_string()),
        "missing shared: {:?}",
        names
    );
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
    assert_eq!(
        found.len(),
        2,
        "expected both real and linked topology dirs: {:?}",
        found
    );
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
    assert_eq!(
        subs.len(),
        3,
        "expected build, dist, target — got {:?}",
        subs
    );
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
