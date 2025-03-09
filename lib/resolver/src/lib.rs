mod context;
mod display;
mod event;
mod function;
mod route;
mod topology;
mod cache;

pub use context::Context;
use compiler::{Topology, TopologyKind};
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


pub async fn resolve(env: &Env, sandbox: &str, topology: &Topology, cache: bool) -> Vec<Topology> {

    let nodes = &topology.nodes;
    let mut xs: Vec<Topology> = vec![];

    let root = topology::resolve(topology, env, sandbox, cache).await;
    xs.push(root);
    for node in nodes {
        let node_t = topology::resolve(&node, env, sandbox, cache).await;
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
        let node_t = topology::resolve(&node, env, sandbox, false).await;
        xs.push(node_t);
    }
    xs
}

pub async fn render(env: &Env, sandbox: &str, topology: &Topology) -> Topology {
    let ctx = Context {
        env: env.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
    };
    let v = serde_json::to_string(topology).unwrap();
    let rendered = ctx.render(&v);
    let t: Topology = serde_json::from_str(&rendered).unwrap();
    t
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

pub fn pprint(topologies: Vec<Topology>, component: Option<String>) -> String {
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
    let t = topology::resolve(&topology, env, &sandbox, true).await;

    let mut fns: Vec<String> = vec![];
    for (_, f) in t.functions {
        fns.push(f.name)
    }

    for node in nodes {
        let node_t = topology::resolve(&node, env, &sandbox, true).await;
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

pub async fn name_of(dir: &str, sandbox: &str, kind: TopologyKind) -> Option<String> {
    let topology = compiler::compile(&dir, false);
    match kind {
        TopologyKind::StepFunction => {
            let nodes = just_nodes(&topology).await;
            let node = nodes.into_iter().nth(0).unwrap();
            Some(node)
        }
        TopologyKind::Function => current_function(sandbox),
        TopologyKind::Evented => None,
    }
}
