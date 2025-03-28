mod context;
mod event;
mod function;
mod route;
mod topology;
pub mod store;

pub use context::Context;
use compiler::Topology;
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

pub async fn read_cached_topology(env_name: &str, namespace: &str, sandbox: &str) -> Option<Topology> {
    let key = store::make_key(namespace, env_name, sandbox);
    store::read_topology(&key).await
}

pub async fn resolve(env: &Env, sandbox: &str, topology: &Topology, cache: bool) -> Topology {

    let maybe_topology = if cache {
        read_cached_topology(&env.name, &topology.namespace, sandbox).await
    } else {
        None
    };

    match maybe_topology {
        Some(t) => t,
        None => {
            let mut root = topology::resolve(topology, env, sandbox).await;
            let nodes = &topology.nodes;
            let mut resolved_nodes: HashMap<String, Topology> = HashMap::new();
            // FIXME: recurse
            for (name, node) in nodes {
                let node_t = topology::resolve(&node, env, sandbox).await;
                resolved_nodes.insert(name.to_string(), node_t);
            }
            root.nodes = resolved_nodes;
            // write it to cache
            let key = store::make_key(&root.namespace, &env.name, sandbox);
            store::write_topology(&key, &root).await;
            root
        }
    }

}

pub async fn resolve_component(env: &Env, sandbox: &str, topology: &Topology, component: &str) -> Topology {

    let mut root = topology::resolve_component(topology, env, sandbox, component).await;
    let mut resolved_nodes: HashMap<String, Topology> = HashMap::new();
    let nodes = &topology.nodes;

    for (name, node) in nodes {
        let node_t = topology::resolve_component(&node, env, sandbox, component).await;
        resolved_nodes.insert(name.to_string(), node_t);
    }
    root.nodes = resolved_nodes;
    root
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

pub fn pprint(t: &Topology, component: Option<String>) -> String {
    let component = u::maybe_string(component, "all");

    match component.as_ref() {
        "functions" => u::pretty_json(&t.functions),
        "flow"      => match &t.flow {
            Some(f) => u::pretty_json(f),
            _       => u::empty(),
        },
        "events"    => u::pretty_json(&t.events),
        "schedules" => u::pretty_json(&t.schedules),
        "routes"    => u::pretty_json(&t.routes),
        "mutations" => u::pretty_json(&t.mutations),
        "basic"     => u::pretty_json(&t.version),
        "all"       => u::pretty_json(&t),
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

    for (_, node) in nodes {
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
