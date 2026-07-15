use compiler::{
    Entity,
    spec::function::{
        FunctionSpec,
        LangRuntime,
        Provider,
        RuntimeSpec,
    },
};
use composer::aws::{
    function::fs::collect_aux_files,
    role::Role,
};
use kit::s;
use std::{
    collections::HashMap,
    fs,
    path::{
        Path,
        PathBuf,
    },
};
use tempfile::TempDir;

fn fake_fspec(name: &str) -> FunctionSpec {
    FunctionSpec {
        name: name.to_string(),
        dir: None,
        description: None,
        namespace: None,
        fqn: None,
        layer_name: None,
        version: None,
        revision: None,
        runtime: None,
        build: None,
        infra: None,
        test: None,
        infra_dir: None,
        tasks: HashMap::new(),
        assets: None,
        targets: None,
        aux_files: None,
        shared: None,
    }
}

fn fake_rspec_no_overrides() -> RuntimeSpec {
    RuntimeSpec {
        lang: LangRuntime::Python310,
        handler: s!("handler.handler"),
        package_type: None,
        provider: Some(Provider::Lambda),
        vars_file: None,
        role_file: None,
        role_name: None,
        role: None,
        uri: None,
        mount_fs: None,
        fs: None,
        snapstart: None,
        layers: vec![],
        extensions: vec![],
        mem: None,
        code: None,
        arch: None,
        network: None,
        port: None,
        microvm: None,
    }
}

fn fake_role_no_path() -> Role {
    Role::default(Entity::Function)
}

fn fake_role_with_path(path: &str) -> Role {
    let mut r = Role::default(Entity::Function);
    r.path = path.to_string();
    r
}

fn mkdir(root: &Path, rel: &str) {
    fs::create_dir_all(root.join(rel)).unwrap();
}

fn mkfile(root: &Path, rel: &str, contents: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, contents).unwrap();
}

/// Conventional `{infra_dir}/roles/{name}.json` must always be in
/// `aux_files`, even when the file doesn't exist on disk. This is
/// what makes the deletion case work.
#[test]
fn runtime_aux_files_includes_conventional_role_path() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    mkdir(root, "infrastructure/tc/foo");
    let infra_dir = root
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();

    let aux = collect_aux_files(&infra_dir, &fake_fspec("myfn"), None, &fake_role_no_path());

    let expected = format!("{}/roles/myfn.json", infra_dir);
    assert!(
        aux.contains(&expected),
        "expected {} in aux_files; got {:?}",
        expected,
        aux
    );
}

/// Conventional `{infra_dir}/vars/{name}.json` must always be in
/// `aux_files`, even when the file doesn't exist on disk.
#[test]
fn runtime_aux_files_includes_conventional_vars_path() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    mkdir(root, "infrastructure/tc/foo");
    let infra_dir = root
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();

    let aux = collect_aux_files(&infra_dir, &fake_fspec("myfn"), None, &fake_role_no_path());

    let expected = format!("{}/vars/myfn.json", infra_dir);
    assert!(
        aux.contains(&expected),
        "expected {} in aux_files; got {:?}",
        expected,
        aux
    );
}

/// When the composer resolves an explicit role override (`r.role`
/// or a custom role_file on disk), `Role.path` carries the resolved
/// path. `collect_aux_files` must surface it so a change to that
/// file flags the function.
#[test]
fn runtime_aux_files_picks_up_explicit_role_override() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    mkdir(root, "infrastructure/tc/foo");
    let infra_dir = root
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();
    let override_path = root.join("shared/role.json").to_str().unwrap().to_string();

    let aux = collect_aux_files(
        &infra_dir,
        &fake_fspec("myfn"),
        Some(&fake_rspec_no_overrides()),
        &fake_role_with_path(&override_path),
    );

    assert!(
        aux.contains(&override_path),
        "expected override path {} in aux_files; got {:?}",
        override_path,
        aux
    );
}

/// When a function has `r.vars_file` set explicitly, that path must
/// also land in `aux_files` (after being absolutized through
/// `follow_path` if it's a `..`-relative override).
#[test]
fn runtime_aux_files_picks_up_explicit_vars_override() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    mkdir(root, "infrastructure/tc/foo");
    let infra_dir = root
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();
    let vars_path = root.join("shared/vars.json").to_str().unwrap().to_string();

    let mut rspec = fake_rspec_no_overrides();
    rspec.vars_file = Some(vars_path.clone());

    let aux = collect_aux_files(
        &infra_dir,
        &fake_fspec("myfn"),
        Some(&rspec),
        &fake_role_no_path(),
    );

    assert!(
        aux.contains(&vars_path),
        "expected vars override {} in aux_files; got {:?}",
        vars_path,
        aux
    );
}

/// When an inherited parent `roles/function.json` exists in the
/// infra ancestry, `collect_aux_files` must record its path.
/// Otherwise a change to a shared parent role wouldn't flag any of
/// the descendant functions that fall back to it.
#[test]
fn runtime_aux_files_picks_up_inherited_parent_role() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    mkdir(root, "infrastructure/tc/foo");
    mkfile(root, "infrastructure/tc/roles/function.json", "{}");
    let infra_dir = root
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();

    let aux = collect_aux_files(&infra_dir, &fake_fspec("myfn"), None, &fake_role_no_path());

    let parent_role = root
        .join("infrastructure/tc/roles/function.json")
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        aux.iter().any(|p| p == &parent_role),
        "expected inherited parent role {} in aux_files; got {:?}",
        parent_role,
        aux
    );
}

/// `collect_aux_files` sorts and dedupes its output.
#[test]
fn runtime_aux_files_dedupes_and_sorts() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    mkdir(root, "infrastructure/tc/foo");
    let infra_dir = root
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();

    // Force a duplicate by setting the override role to the same
    // path as the conventional location.
    let conventional = format!("{}/roles/myfn.json", infra_dir);

    let aux = collect_aux_files(
        &infra_dir,
        &fake_fspec("myfn"),
        Some(&fake_rspec_no_overrides()),
        &fake_role_with_path(&conventional),
    );

    let mut sorted = aux.clone();
    sorted.sort();
    assert_eq!(aux, sorted, "aux_files must be sorted");

    let count = aux.iter().filter(|p| **p == conventional).count();
    assert_eq!(count, 1, "duplicate path was not deduped: {:?}", aux);
}

/// Path-normalization byte-identity invariant: the path emitted by
/// `collect_aux_files` for a deleted role file must byte-match the
/// path the differ's `build_diff_set` produces for the same deleted
/// file. If this drifts, the deletion case silently breaks in
/// production.
///
/// **Treat any failure here as a release blocker.** The fix is
/// almost always to add a `lexically_normalize` / `canonicalize_or_join_against_root`
/// helper applied uniformly on both sides.
#[test]
fn composer_aux_path_byte_matches_diff_set_path_for_deleted_role_file() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();
    mkdir(repo_root, "infrastructure/tc/foo");
    mkdir(repo_root, "topologies/foo/myfn");

    // Mirror what the composer produces for an `infra_dir`: an
    // absolute, canonical path under the repo root.
    let infra_dir = repo_root
        .canonicalize()
        .unwrap()
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();

    let composer_path: String = collect_aux_files(
        &infra_dir,
        &fake_fspec("myfn"),
        Some(&fake_rspec_no_overrides()),
        &fake_role_no_path(),
    )
    .into_iter()
    .find(|p| p.ends_with("roles/myfn.json"))
    .expect("conventional role path must always be emitted");

    // Mirror `build_diff_set`: canonicalize the repo root, join
    // the rel path, fall back to the logical join when the file
    // doesn't exist (deletion case).
    let rel = "infrastructure/tc/foo/roles/myfn.json";
    let canonical_root = repo_root.canonicalize().unwrap();
    let joined = canonical_root.join(rel);
    let diff_path: PathBuf = joined.canonicalize().unwrap_or(joined);

    assert_eq!(
        PathBuf::from(&composer_path),
        diff_path,
        "composer aux path and differ diff-set path MUST be \
         byte-identical for the deletion case to fire"
    );
}

/// Same byte-identity invariant as above, but for the conventional
/// vars file.
#[test]
fn composer_aux_path_byte_matches_diff_set_path_for_deleted_vars_file() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();
    mkdir(repo_root, "infrastructure/tc/foo");

    let infra_dir = repo_root
        .canonicalize()
        .unwrap()
        .join("infrastructure/tc/foo")
        .to_str()
        .unwrap()
        .to_string();

    let composer_path: String = collect_aux_files(
        &infra_dir,
        &fake_fspec("myfn"),
        Some(&fake_rspec_no_overrides()),
        &fake_role_no_path(),
    )
    .into_iter()
    .find(|p| p.ends_with("vars/myfn.json"))
    .expect("conventional vars path must always be emitted");

    let rel = "infrastructure/tc/foo/vars/myfn.json";
    let canonical_root = repo_root.canonicalize().unwrap();
    let joined = canonical_root.join(rel);
    let diff_path: PathBuf = joined.canonicalize().unwrap_or(joined);

    assert_eq!(
        PathBuf::from(&composer_path),
        diff_path,
        "composer aux vars path must byte-match differ diff-set path"
    );
}
