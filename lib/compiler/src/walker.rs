use std::collections::HashMap;
use crate::{Entity, TopologySpec, FunctionSpec, TestSpec,
            TopologyKind, RoleSpec};
use std::path::Path;
use kit as u;
use kit::*;

use crate::spec::{template, state};
use configurator::Config;
use walkdir::WalkDir;
use serde_json::Value;

fn flow_of(f: &Option<Value>, s: &Option<Value>) -> Option<Value> {
    if f.is_some() {
        f.clone()
    } else {
        s.clone()
    }
}

fn find_kind(spec: &TopologySpec)  -> TopologyKind {

    let TopologySpec { kind, flow, states,
                       functions,
                       mutations, routes, .. } = spec;

    let flow = flow_of(flow, states);

    match kind {
        Some(k) => k.clone(),
        None => match flow {
            Some(_) => TopologyKind::StepFunction,
            None => {
                if mutations.is_some() {
                    return TopologyKind::Graphql;
                } else if routes.is_some() {
                    return TopologyKind::Routed;
                } else if functions.is_some() {
                    return TopologyKind::Function;
                } else {
                    return TopologyKind::Evented;
                }
            }
        },
    }
}

fn make_roles(spec: &TopologySpec, namespace: &str, fqn: &str, infra_dir: &str) -> HashMap<String, RoleSpec> {

    let TopologySpec { functions, mutations, routes,
                       events, states, .. } = spec;

    let fns = match functions {
        Some(xs) => xs,
        None => &HashMap::new()
    };

    let mut h: HashMap<String, RoleSpec> = HashMap::new();
    for (_, f) in fns {
        if let Some(runtime) = &f.runtime {
            let role_spec = runtime.role_spec.clone();
            if let Some(r) = role_spec {
                if r.kind.to_str() == "override" {
                    h.insert(r.name.clone(), r);
                }
            }
        }
    }

    if let Some(_f) = states {
        let role = state::make_role(infra_dir, namespace, fqn);
        h.insert(s!("state"), role.clone());
    }

    let mut entities: Vec<Entity> = vec![];

    if let Some(m) = mutations {
        if m.resolvers.len() > 0 {
            entities.push(Entity::Mutation);
        }
    }

    if let Some(r) = routes {
        if r.len() > 0 {
            entities.push(Entity::Route);
        }
    }

    if let Some(e) = events {
        if e.len() > 0 {
            entities.push(Entity::Event);
        }
    }

    if let Some(_f) = states {
        entities.push(Entity::State);
    }

    for b in entities {
        let r = match std::env::var("TC_LEGACY_ROLES") {
            Ok(_) => RoleSpec::provided_by_entity(b),
            Err(_) => RoleSpec::default(b),
        };
        h.insert(r.name.clone(), r);
    }
    h
}

fn relative_root_path(dir: &str) -> (String, String) {
    let root = u::split_first(&dir, "/topologies/");
    let next = u::second(&dir, "/topologies/");
    (root, next)
}

fn as_infra_dir(given_infra_dir: Option<String>, topology_dir: &str) -> String {
    match given_infra_dir {
        Some(d) => d,
        None => {
            let (root, next) = relative_root_path(topology_dir);
            let s = next.replace("_", "-");
            format!("{root}/infrastructure/tc/{s}")
        }
    }
}

pub fn is_topology_dir(dir: &str) -> bool {
    let topology_file = format!("{}/topology.yml", dir);
    Path::new(&topology_file).exists()
}

fn parent_topology_file(dir: &str) -> Option<String> {
    let paths = vec![
        u::absolutize(dir, "../topology.yml"),
        u::absolutize(dir, "../../topology.yml"),
        u::absolutize(dir, "../../../topology.yml"),
        u::absolutize(dir, "../../../../topology.yml"),
        s!("../topology.yml"),
        s!("../../topology.yml"),
        s!("../../../topology.yml"),
        s!("../../../../topology.yml"),
    ];
    u::any_path(paths)
}

pub fn is_root_topology(spec_file: &str) -> bool {
    let spec = TopologySpec::new(spec_file);
    if let Some(given_root_dirs) = &spec.nodes.dirs {
        !given_root_dirs.is_empty()
    } else {
        spec.nodes.root.is_some()
    }
}

pub fn is_relative_topology_dir(dir: &str) -> bool {
    let topology_file = parent_topology_file(dir);
    match topology_file {
        Some(file) => {
            if is_root_topology(&file) {
                false
            } else {
                Path::new(&file).exists()
            }
        }
        None => false,
    }
}

fn is_standalone_function_dir(dir: &str) -> bool {
    let function_file = "function.yml";
    let function_file_json = "function.json";
    let topology_file = "topology.yml";
    let parent_file = match parent_topology_file(dir) {
        Some(file) => file,
        None => u::empty(),
    };
    if is_root_topology(&parent_file) {
        return true;
    } else {
        (u::file_exists(function_file) || u::file_exists(function_file_json))
            && !u::file_exists(topology_file)
            && !u::file_exists(&parent_file)
            || u::file_exists("handler.rb")
            || u::file_exists("handler.py")
            || u::file_exists("main.go")
            || u::file_exists("Cargo.toml")
            || u::file_exists("handler.janet")
            || u::file_exists("handler.clj")
            || u::file_exists("handler.js")
            || u::file_exists("main.janet")
    }
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    u::file_exists(function_file) && u::file_exists(topology_file)
}

fn is_shared(uri: Option<String>) -> bool {
    match uri {
        Some(p) => p.starts_with("."),
        None => false,
    }
}

fn abs_shared_dir(root_dir: &str, uri: Option<String>) -> String {
    match uri {
        Some(p) => u::absolute_dir(&root_dir, &p),
        None => panic!("Shared uri not specified"),
    }
}

fn intern_functions(
    root_namespace: &str,
    infra_dir: &str,
    spec: &TopologySpec

) -> HashMap<String, FunctionSpec> {

    let inline_fns = match &spec.functions {
        Some(f) => f,
        None => &HashMap::new(),
    };

    let mut fns: HashMap<String, FunctionSpec> = HashMap::new();
    let root_dir = &spec.dir.clone().unwrap();

    for (name, f) in inline_fns {
        if is_shared(f.uri.clone()) {
            let abs_dir = abs_shared_dir(root_dir, f.uri.clone());
            let _namespace = match &f.fqn {
                Some(_) => &spec.name,
                None => root_namespace,
            };
            let mut fs = FunctionSpec::new(&abs_dir);
            fs.infra_dir = Some(s!(infra_dir));
            fns.insert(s!(name), fs);

        } else {
            let dir = format!("{}/{}", root_dir, name);
            let namespace = &spec.name;
            let fs = f.intern(namespace, &dir, infra_dir, &name);
            fns.insert(s!(name), fs);
        }
    }
    fns
}

fn is_inferred_dir(dir: &str) -> bool {
    u::path_exists(dir, "handler.rb")
        || u::path_exists(dir, "handler.py")
        || u::path_exists(dir, "main.go")
        || u::path_exists(dir, "Cargo.toml")
        || u::path_exists(dir, "handler.janet")
        || u::path_exists(dir, "handler.clj")
        || u::path_exists(dir, "handler.js")
        || u::path_exists(dir, "main.janet")
}

fn function_dirs(dir: &str) -> Vec<String> {
    let known_roots = vec!["resolvers", "functions", "backend"];
    let mut xs: Vec<String> = vec![];
    let dirs = u::list_dirs(dir);
    for root in known_roots {
        let mut xm = u::list_dirs(root);
        xs.append(&mut xm)
    }
    for d in dirs {
        if path_exists(&d, "function.yml")
            || path_exists(&d, "function.json")
            || is_inferred_dir(&d)
        {
            xs.push(d.to_string())
        }
    }
    xs
}


fn ignore_function(dir: &str, root_dir: &str) -> bool {
    let ignore_file = u::path_of(root_dir, ".tcignore");
    if dir.contains(".circleci")
        || dir.contains(".git")
        || dir.contains(".vendor")
        || dir.contains(".venv")
        || dir.contains(".env")
        || dir.contains("node_modules")
        || dir.ends_with("entities")
        || dir.ends_with("states")
        || dir.ends_with("topology")
    {
        return true;
    }
    if u::file_exists(&ignore_file) {
        let globs = u::readlines(&ignore_file);
        for g in globs {
            if u::is_dir(&g) && dir.ends_with(&g) {
                return true;
            } else {
                continue;
            }
        }
        return false;
    } else {
        false
    }
}

fn current_function(dir: &str, infra_dir: &str, _namespace: &str) -> HashMap<String, FunctionSpec> {
    let mut functions: HashMap<String, FunctionSpec> = HashMap::new();
    if u::is_dir(dir) && !dir.starts_with(".") {
        let mut fs = FunctionSpec::new(dir);
        fs.infra_dir = Some(s!(infra_dir));
        functions.insert(s!(fs.name), fs);
    }
    functions
}

fn should_ignore_node(
    root_dir: &str,
    ignore_nodes: Option<Vec<String>>,
    topology_dir: &str,
) -> bool {
    let ignore_file = u::path_of(root_dir, ".tcignore");
    if u::file_exists(&ignore_file) {
        let globs = u::readlines(&ignore_file);
        for g in globs {
            let gdir = format!("{}/{}", root_dir, &g);
            if topology_dir.ends_with(&g) || topology_dir.contains(&gdir) {
                return true;
            } else {
                continue;
            }
        }
        return false;
    } else {
        for node in ignore_nodes.unwrap() {
            let abs_path = format!("{root_dir}/{node}");
            if &abs_path == topology_dir {
                return true;
            }
            if topology_dir.starts_with(&abs_path) {
                return true;
            }
            return false;
        }
        return false;
    }
}


fn discover_functions(
    dir: &str,
    infra_dir: &str,
    spec: &TopologySpec,
) -> HashMap<String, FunctionSpec> {

    let mut functions: HashMap<String, FunctionSpec> = HashMap::new();
    let dirs = function_dirs(dir);

    for d in dirs {
        tracing::debug!("function {}", d);
        if u::is_dir(&d) && !ignore_function(&d, dir) {
            let mut fs = FunctionSpec::new(&d);
            fs.namespace = Some(s!(spec.name));
            fs.infra_dir = Some(s!(infra_dir));
            functions.insert(fs.name.clone(), fs);
        }
    }
    functions
}

fn discover_leaf_nodes(
    root_ns: &str,
    root_dir: &str,
    dir: &str,
    s: &TopologySpec,
) -> HashMap<String, TopologySpec> {
    let ignore_nodes = &s.nodes.ignore;

    let mut nodes: HashMap<String, TopologySpec> = HashMap::new();
    if is_topology_dir(dir) {
        if !should_ignore_node(root_dir, ignore_nodes.clone(), dir) {
            let f = format!("{}/topology.yml", dir);
            let spec = TopologySpec::new(&f);
            let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
            let mut functions = discover_functions(dir, &infra_dir, &spec);
            let interned = intern_functions(root_ns, &infra_dir, &spec);
            functions.extend(interned);
            let node = make(root_dir, dir, &spec, functions, HashMap::new());
            nodes.insert(spec.name.to_string(), node);
        }
    }
    nodes
}

fn make_nodes(root_dir: &str, spec: &TopologySpec) -> HashMap<String, TopologySpec> {
    let ignore_nodes = &spec.nodes.ignore;
    let mut nodes: HashMap<String, TopologySpec> = HashMap::new();
    for entry in WalkDir::new(root_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
    {
        let p = entry.path().to_string_lossy();
        if is_topology_dir(&p) && root_dir != p.clone() {
            if !should_ignore_node(root_dir, ignore_nodes.clone(), &p) {
                let f = format!("{}/topology.yml", &p);
                let spec = TopologySpec::new(&f);
                tracing::debug!("node {}", &spec.name);
                let infra_dir = as_infra_dir(spec.infra.to_owned(), &p);
                let mut functions = discover_functions(&p, &infra_dir, &spec);
                let interned = intern_functions(&p, &infra_dir, &spec);
                functions.extend(interned);
                let leaf_nodes = discover_leaf_nodes(&spec.name, root_dir, &p, &spec);
                let node = make(root_dir, &p, &spec, functions, leaf_nodes);
                nodes.insert(spec.name.to_string(), node);
            }
        }
    }
    nodes
}

fn make_test(
    t: Option<HashMap<String, TestSpec>>,
    fns: &HashMap<String, FunctionSpec>,
) -> HashMap<String, TestSpec> {
    let mut tspecs = match t {
        Some(spec) => spec,
        None => HashMap::new(),
    };
    for (fname, f) in fns {
        if let Some(test) = &f.test {
            for (name, mut tspec) in test.clone() {
                tspec.entity = Some(format!("function/{}", &fname));
                tspecs.insert(name.to_string(), tspec.clone());
            }
        }
    }
    tspecs
}

fn make_relative(dir: &str) -> TopologySpec {
    let f = match parent_topology_file(dir) {
        Some(file) => file,
        None => format!("../topology.yml"),
    };

    let spec = TopologySpec::new(&f);
    let namespace = &spec.name;
    let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
    let mut function = FunctionSpec::new(dir);
    function.infra_dir = Some(infra_dir);
    function.namespace = Some(s!(namespace));

    let mut fns: HashMap<String, FunctionSpec> = HashMap::new();
    fns.insert(function.name.to_string(), function);

    let nodes = HashMap::new();
    make(dir, dir, &spec, fns, nodes)
}

fn make_standalone(dir: &str) -> TopologySpec {
    let mut function = FunctionSpec::new(dir);
    let namespace = &function.name.clone();

    function.infra_dir = Some(s!(dir));

    let mut fns: HashMap<String, FunctionSpec> = HashMap::new();
    fns.insert(function.name.to_string(), function);

    TopologySpec::standalone(dir, &namespace, fns)
}

fn make(
    _root_dir: &str,
    dir: &str,
    spec: &TopologySpec,
    functions: HashMap<String, FunctionSpec>,
    nodes: HashMap<String, TopologySpec>,
) -> TopologySpec {

    let config = Config::new();

    let mut functions = functions;
    let namespace = spec.name.to_owned();
    let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
    tracing::debug!(
        "node-infra-dir {:?}, {} {}",
        &spec.infra,
        &spec.name,
        &infra_dir
    );
    let interned = intern_functions(&namespace, &infra_dir, &spec);
    functions.extend(interned);

    let version = u::current_semver(&namespace);
    let fqn = template::topology_fqn(&namespace);
    let roles = make_roles(&spec, &namespace, &fqn, &infra_dir);
    let kind = find_kind(spec);

    let mut ts = spec.clone();
    ts.kind = Some(kind);
    ts.fqn = Some(fqn);
    ts.children = Some(nodes);
    ts.version = Some(version);
    ts.tests = Some(make_test(spec.tests.clone(), &functions));
    ts.functions = Some(functions);
    ts.infra = Some(u::gdir(&infra_dir));
    ts.roles = Some(roles);
    ts.config = Some(config);
    ts.clone()
}

pub fn walk(spec: &TopologySpec, recursive: bool) -> TopologySpec {
    let dir = match &spec.dir {
        Some(d) => d,
        None => panic!("No dir found")
    };

    if is_singular_function_dir() {
        let infra_dir = as_infra_dir(spec.infra.to_owned(), &spec.name);
        let functions = current_function(dir, &infra_dir, &spec.name);
        make(dir, dir, &spec, functions, HashMap::new())

    } else if is_topology_dir(dir) {
        let infra_dir = as_infra_dir(spec.infra.to_owned(), dir);
        tracing::debug!("Infra dir: {}  {}", &spec.name, &infra_dir);
        let children = if recursive {
            make_nodes(dir, &spec)
        } else {
            HashMap::new()
        };
        tracing::debug!("Discovering functions {}", dir);
        let functions = discover_functions(dir, &infra_dir, &spec);
        make(dir, dir, &spec, functions, children)
    } else if is_relative_topology_dir(dir) {
            make_relative(dir)
    } else if is_standalone_function_dir(dir) {
        make_standalone(dir)
    } else {
        println!("{}", dir);
        std::panic::set_hook(Box::new(|_| {
            println!("No topology.yml or function.json found. Inference failed");
        }));
        panic!("Don't know what to do");
    }
}
