mod aws;
pub mod cache;
mod context;
mod event;
mod function;
mod pool;
mod topology;

use authorizer::Auth;
use composer::{
    Entity,
    Topology,
};
pub use context::Context;
use std::collections::HashMap;

pub fn maybe_sandbox(s: Option<String>) -> String {
    match s {
        Some(sandbox) => sandbox,
        None => match std::env::var("TC_SANDBOX") {
            Ok(e) => e,
            Err(_) => panic!("Please specify sandbox or set TC_SANDBOX env variable"),
        },
    }
}

pub async fn render(auth: &Auth, sandbox: &str, topology: &Topology) -> Topology {
    let ctx = Context {
        auth: auth.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
        config: topology.config.to_owned(),
    };
    let v = serde_json::to_string(topology).unwrap();
    let rendered = ctx.render(&v);
    let t: Topology = serde_json::from_str(&rendered).unwrap();
    t
}

pub async fn read_cached_topology(
    env_name: &str,
    namespace: &str,
    sandbox: &str,
) -> Option<Topology> {
    let key = cache::make_key(namespace, env_name, sandbox);
    cache::read_topology(&key).await
}

pub async fn resolve(
    auth: &Auth,
    sandbox: &str,
    topology: &Topology,
    cache: bool,
    diff: bool
) -> Topology {
    let maybe_topology = if cache {
        read_cached_topology(&auth.name, &topology.namespace, sandbox).await
    } else {
        None
    };

    match maybe_topology {
        Some(t) => t,
        None => {
            let mut root = topology::resolve(
                topology, topology, auth, sandbox, diff
            ).await;
            let nodes = &topology.nodes;
            let mut resolved_nodes: HashMap<String, Topology> = HashMap::new();
            // FIXME: recurse
            for (name, node) in nodes {
                let node_t = topology::resolve(&root, &node, auth, sandbox, diff).await;
                resolved_nodes.insert(name.to_string(), node_t);
            }
            root.nodes = resolved_nodes;
            // write it to cache
            let key = cache::make_key(&root.namespace, &auth.name, sandbox);
            cache::write_topology(&key, &root).await;
            root
        }
    }
}

async fn resolve_entity(
    auth: &Auth,
    sandbox: &str,
    topology: &Topology,
    entity: &Entity,
    diff: bool
) -> Topology {
    let mut root = topology::resolve_entity(topology, auth, sandbox, entity, diff).await;
    let mut resolved_nodes: HashMap<String, Topology> = HashMap::new();
    let nodes = &topology.nodes;

    for (name, node) in nodes {
        let node_t = topology::resolve_entity(&node, auth, sandbox, entity, diff).await;
        resolved_nodes.insert(name.to_string(), node_t);
    }
    root.nodes = resolved_nodes;
    root
}

pub async fn try_resolve(
    auth: &Auth,
    sandbox: &str,
    topology: &Topology,
    maybe_entity: &Option<String>,
    cache: bool,
    diff: bool
) -> Topology {
    match maybe_entity {
        Some(e) => {
            let (entity, _) = Entity::as_entity_component(&e);
            resolve_entity(auth, sandbox, topology, &entity, diff).await
        }
        None => resolve(auth, sandbox, topology, cache, diff).await,
    }
}
