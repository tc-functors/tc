use composer::{Topology};
use composer::topology::*;
use std::{
    fs,
    os::unix::fs::symlink,
};
use kit::s;
use tempfile::TempDir;

#[test]
fn should_ignore_node_matches_when_root_reached_via_symlink() {
    let outer = TempDir::new().unwrap();
    let real = outer.path().join("real");
    fs::create_dir_all(real.join("ignore_me")).unwrap();
    fs::create_dir_all(real.join("keep_me")).unwrap();
    let canonical_real = real.canonicalize().unwrap();

    let alias = outer.path().join("link");
    symlink(&real, &alias).unwrap();

    let alias_root = alias.to_str().unwrap();
    let canonical_target = canonical_real.join("ignore_me");
    let canonical_target_str = canonical_target.to_str().unwrap();
    let canonical_keep = canonical_real.join("keep_me");
    let canonical_keep_str = canonical_keep.to_str().unwrap();

    let ignore = Some(vec!["ignore_me".to_string()]);
    assert!(
        should_ignore_node(alias_root, ignore.clone(), canonical_target_str),
        "ignore rule should fire even though root_dir uses an aliased path"
    );
    assert!(
        !should_ignore_node(alias_root, ignore, canonical_keep_str),
        "non-matching dir should not be ignored"
    );
}

fn write_shared_function(dir: &std::path::Path, name: &str, fqn: &str) {
    fs::create_dir_all(dir).unwrap();
    fs::write(
        dir.join("function.yml"),
        format!(
            "name: {name}\n\
             fqn: {fqn}\n\
             runtime:\n  \
             lang: python3.10\n  \
             handler: handler.handler\n  \
             package_type: zip\n  \
             layers: []\n\
             build:\n  \
             kind: Code\n  \
             command: echo build\n"
        ),
    )
        .unwrap();
    fs::write(dir.join("handler.py"), "").unwrap();
}

fn write_topology_yml(dir: &std::path::Path, contents: &str) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("topology.yml"), contents).unwrap();
}

#[test]
fn compose_recursive_dedups_shared_functions_to_root() {
    let outer = TempDir::new().unwrap();
    let root = outer.path();

    write_topology_yml(root, "name: shared-dedup-parent\nkind: step-function\n");

    write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");
    write_shared_function(&root.join("shared/bar"), "bar", "shared_bar");

    write_topology_yml(
        &root.join("a"),
        "name: child-a\nkind: step-function\nfunctions:\n  foo:\n    uri: ../shared/foo\n",
    );
    write_topology_yml(
        &root.join("b"),
        "name: child-b\nkind: step-function\nfunctions:\n  foo:\n    uri: ../shared/foo\n",
    );
    write_topology_yml(
        &root.join("c"),
        "name: child-c\n\
         kind: step-function\n\
         functions:\n  \
         foo:\n    uri: ../shared/foo\n  \
         bar:\n    uri: ../shared/bar\n",
    );
    write_shared_function(&root.join("c/local"), "local", "child_c_local");

    let topology = Topology::new(root.to_str().unwrap(), true, false);

    assert!(
        topology.functions.contains_key("foo"),
        "shared function `foo` must be promoted to root.functions; root has {:?}",
        topology.functions.keys().collect::<Vec<_>>()
    );
    assert!(
        topology.functions.contains_key("bar"),
        "shared function `bar` must be promoted to root.functions; root has {:?}",
        topology.functions.keys().collect::<Vec<_>>()
    );
    assert!(
        topology
            .functions
            .get("foo")
            .map(|f| f.shared)
            .unwrap_or(false),
        "promoted `foo` retains shared = true"
    );
    assert!(
        topology
            .functions
            .get("bar")
            .map(|f| f.shared)
            .unwrap_or(false),
        "promoted `bar` retains shared = true"
    );

    fn collect_offenders(
        t: &Topology,
        path: String,
        keys: &[&'static str],
        out: &mut Vec<(String, &'static str)>,
    ) {
        for k in keys {
            if t.functions.contains_key(*k) {
                out.push((path.clone(), *k));
            }
        }
        for (name, child) in &t.nodes {
            collect_offenders(child, format!("{path}/{name}"), keys, out);
        }
    }

    let mut offenders: Vec<(String, &'static str)> = Vec::new();
    for (name, child) in &topology.nodes {
        collect_offenders(child, name.clone(), &["foo", "bar"], &mut offenders);
    }
    assert!(
        offenders.is_empty(),
        "no descendant should retain a shared function; found: {:?}",
        offenders
    );

    let child_namespaces: std::collections::BTreeSet<&String> = topology.nodes.keys().collect();
    assert!(
        child_namespaces.contains(&s!("child-a")),
        "child-a missing; root.nodes = {:?}",
        child_namespaces
    );
    assert!(
        child_namespaces.contains(&s!("child-b")),
        "child-b missing; root.nodes = {:?}",
        child_namespaces
    );
    assert!(
        child_namespaces.contains(&s!("child-c")),
        "child-c missing; root.nodes = {:?}",
        child_namespaces
    );

    let child_c = topology.nodes.get("child-c").expect("child-c node present");
    let local = child_c
        .functions
        .get("local")
        .expect("child-c retains its own `local` function after promotion");
    assert!(
        !local.shared,
        "child-c's own `local` function must have shared = false"
    );

    for (fname, f) in &topology.functions {
        if f.runtime.role.kind.to_str() != "provided" {
            assert!(
                topology.roles.contains_key(&f.runtime.role.name),
                "role `{}` for function `{}` must be present in root.roles \
                 after promotion; root.roles keys = {:?}",
                &f.runtime.role.name,
                fname,
                topology.roles.keys().collect::<Vec<_>>()
            );
        }
    }
    fn assert_roles_match_functions(t: &Topology, path: &str) {
        for (rname, _) in &t.roles {
            let owned_by_function = t.functions.values().any(|f| &f.runtime.role.name == rname);
            let owned_by_flow = t
                .flow
                .as_ref()
                .map(|fl| &fl.role.name == rname)
                .unwrap_or(false);
            assert!(
                owned_by_function || owned_by_flow,
                "stale role `{}` in {}/roles after promotion (no matching \
                 function or flow); functions = {:?}",
                rname,
                path,
                t.functions.keys().collect::<Vec<_>>()
            );
        }
        for (name, child) in &t.nodes {
            assert_roles_match_functions(child, &format!("{path}/{name}"));
        }
    }
    for (name, child) in &topology.nodes {
        assert_roles_match_functions(child, name);
    }
}

#[test]
fn intern_marks_relative_uri_imports_as_shared() {
    let outer = TempDir::new().unwrap();
    let root = outer.path();
    write_topology_yml(root, "name: parent\nkind: step-function\n");
    write_shared_function(&root.join("shared/x"), "x", "shared_x");
    write_topology_yml(
        &root.join("a"),
        "name: child-a\nkind: step-function\nfunctions:\n  x:\n    uri: ../shared/x\n",
    );

    let topology = Topology::new(root.to_str().unwrap(), true, false);
    let promoted = topology
        .functions
        .get("x")
        .expect("relatively-imported function must end up at root");
    assert!(
        promoted.shared,
        "relative-uri import must have shared = true after promotion"
    );
}

#[test]
fn root_declaring_shared_function_keeps_it_in_place() {
    let outer = TempDir::new().unwrap();
    let root = outer.path();
    write_shared_function(&root.join("shared/x"), "x", "shared_x");
    write_topology_yml(
        root,
        "name: root-shared\nkind: step-function\nfunctions:\n  x:\n    uri: ./shared/x\n",
    );

    let topology = Topology::new(root.to_str().unwrap(), true, false);
    assert!(
        topology.functions.contains_key("x"),
        "root's own shared function must remain in root.functions"
    );
    assert!(
        topology.functions.get("x").unwrap().shared,
        "root's own shared function retains shared = true"
    );
}

#[test]
fn same_source_different_keys_both_promoted() {
    let outer = TempDir::new().unwrap();
    let root = outer.path();
    write_topology_yml(root, "name: parent\nkind: step-function\n");
    write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");
    write_topology_yml(
        &root.join("a"),
        "name: child-a\nkind: step-function\nfunctions:\n  foo:\n    uri: ../shared/foo\n",
    );
    write_topology_yml(
        &root.join("b"),
        "name: child-b\nkind: step-function\nfunctions:\n  my_foo:\n    uri: ../shared/foo\n",
    );

    let topology = Topology::new(root.to_str().unwrap(), true, false);

    assert!(
        topology.functions.contains_key("foo"),
        "key `foo` must be promoted"
    );
    assert!(
        topology.functions.contains_key("my_foo"),
        "key `my_foo` must be promoted (same source, different local name)"
    );
    assert_eq!(
        topology.functions.get("foo").unwrap().dir,
        topology.functions.get("my_foo").unwrap().dir,
        "both keys reference the same source dir"
    );
}

#[test]
fn deep_nesting_shared_functions_promoted() {
    let outer = TempDir::new().unwrap();
    let root = outer.path();
    write_topology_yml(root, "name: parent\nkind: step-function\n");
    write_shared_function(&root.join("shared/foo"), "foo", "shared_foo");

    // Two levels: root -> mid -> leaf, leaf imports shared function
    write_topology_yml(&root.join("mid"), "name: mid\nkind: step-function\n");
    write_topology_yml(
        &root.join("mid/leaf"),
        "name: leaf\nkind: step-function\nfunctions:\n  foo:\n    uri: ../../shared/foo\n",
    );

    let topology = Topology::new(root.to_str().unwrap(), true, false);

    assert!(
        topology.functions.contains_key("foo"),
        "shared function from deep nesting must be promoted to root"
    );

    fn has_shared_function(t: &Topology, key: &str) -> bool {
        if t.functions.contains_key(key) {
            return true;
        }
        t.nodes
            .values()
            .any(|child| has_shared_function(child, key))
    }
    for child in topology.nodes.values() {
        assert!(
            !has_shared_function(child, "foo"),
            "no descendant should retain the shared function"
        );
    }
}

#[test]
fn non_recursive_mode_does_not_promote() {
    let outer = TempDir::new().unwrap();
    let root = outer.path();
    write_shared_function(&root.join("shared/x"), "x", "shared_x");
    write_topology_yml(
        root,
        "name: non-recursive\nkind: step-function\nfunctions:\n  x:\n    uri: ./shared/x\n",
    );

    let topology = Topology::new(root.to_str().unwrap(), false, false);

    assert!(
        topology.functions.contains_key("x"),
        "function present in non-recursive mode"
    );
    assert!(
        topology.functions.get("x").unwrap().shared,
        "shared flag is set even in non-recursive mode (marking is unconditional)"
    );
    assert!(
        topology.nodes.is_empty(),
        "non-recursive mode has no child nodes"
    );
}
