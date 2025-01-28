pub mod context;
mod display;
mod event;
mod function;
mod route;
mod topology;

pub use context::Context;
use compiler::{Topology};
use aws::Env;
use kit as u;
use std::collections::HashMap;

pub fn maybe_sandbox(s: Option<String>) -> String {
    match s {
        Some(sandbox) => sandbox,
        None => match std::env::var("TC_SANDBOX") {
            Ok(e) => e,
            Err(_) => panic!("Please specify sandbox or set TC_SANDBOX env variable")
        }
    }
}


pub async fn resolve(env: &Env, sandbox: &str, topology: &Topology) -> Vec<Topology> {

    let nodes = &topology.nodes;
    let mut xs: Vec<Topology> = vec![];

    let root = topology::resolve(topology, env, sandbox).await;
    xs.push(root);
    for node in nodes {
        let node_t = topology::resolve(&node, env, sandbox).await;
        xs.push(node_t);
    }
    xs
}

pub async fn resolve_component(env: &Env, sandbox: &str, topology: &Topology, component: &str) -> Vec<Topology> {

    let nodes = &topology.nodes;
    let mut xs: Vec<Topology> = vec![];

    let root = topology::resolve_component(topology, env, sandbox, component).await;
    xs.push(root);
    for node in nodes {
        let node_t = topology::resolve(&node, env, sandbox).await;
        xs.push(node_t);
    }
    xs
}

pub async fn just_nodes(topology: &Topology) -> Vec<String> {
    let mut nodes: Vec<String> = vec![];
    let root = &topology.fqn;
    nodes.push(root.to_string());
    for node in &topology.nodes {
        nodes.push(node.fqn.to_string());
    }
    nodes
}

pub fn render(topologies: Vec<Topology>, component: Option<String>) -> String {
    let t = topologies.clone().into_iter().nth(0).unwrap();
    let component = u::maybe_string(component, "all");

    match component.as_ref() {
        "functions" => u::pretty_json(t.functions),
        "flow"      => match t.flow {
            Some(f) => u::pretty_json(f),
            _       => u::empty(),
        },
        "layers"    => display::render_layers(&topologies),
        "events"    => u::pretty_json(t.events),
        "schedules" => u::pretty_json(t.schedules),
        "routes"    => u::pretty_json(t.routes),
        "mutations" => u::pretty_json(t.mutations),
        "basic"     => u::pretty_json(t.version),
        "all"       => u::pretty_json(topologies),
        _           => u::empty()
    }
}

pub async fn functions(dir: &str, env: &Env, sandbox: Option<String>) -> Vec<String> {
    let topology = compiler::compile(&dir, true);
    let nodes = &topology.nodes;

    let sandbox = maybe_sandbox(sandbox);
    let t = topology::resolve(&topology, env, &sandbox).await;

    let mut fns: Vec<String> = vec![];
    for (_, f) in t.functions {
        fns.push(f.name)
    }

    for node in nodes {
        let node_t = topology::resolve(&node, env, &sandbox).await;
        for (_, f) in node_t.functions {
            fns.push(f.name)
        }
    }
    fns
}

pub fn current_function(sandbox: &str) -> Option<String> {
    let dir = u::pwd();
    let topology = compiler::compile(&dir, false);

    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);

    for (cdir, f) in topology.functions {
        if &cdir == &dir {
            return Some(u::stencil(&f.fqn, table));
        }
    }
    None
}

pub async fn name_of(dir: &str, sandbox: &str, kind: &str) -> Option<String> {
    let topology = compiler::compile(&dir, false);
    match kind {
        "step-function" => {
            let nodes = just_nodes(&topology).await;
            let node = nodes.into_iter().nth(0).unwrap();
            Some(node)
        }
        "lambda" | "function" => current_function(sandbox),
        _ => None,
    }
}
