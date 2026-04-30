pub mod cache;
mod context;
mod event;
pub mod function;
mod memo;
mod pool;
mod topology;
use compiler::Entity;
use composer::Topology;
pub use context::Context;
pub use function::Root;
use futures::stream::{
    self,
    StreamExt,
};
use provider::Auth;
use std::collections::HashMap;

/// Bound on concurrent in-flight `topology::resolve` / `resolve_runtime`
/// awaits. Default 16 keeps us well under per-account API throttle
/// limits (Lambda ListLayerVersions: 15 TPS, APIGateway GetApis: 10 TPS,
/// SSM GetParameter: 40 TPS) once the per-call caches in
/// [`function`] are in place — those caches collapse the duplicated
/// lookups long before they hit the wire. Override via
/// `TC_RESOLVE_CONCURRENCY=N` for repos that outgrow the default.
pub(crate) fn resolve_concurrency() -> usize {
    std::env::var("TC_RESOLVE_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(16)
}

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
    force: bool,
) -> Topology {
    let maybe_topology = if cache {
        read_cached_topology(&auth.name, &topology.namespace, sandbox).await
    } else {
        None
    };

    match maybe_topology {
        Some(t) => t,
        None => {
            let mut root = topology::resolve(topology, topology, auth, sandbox, force).await;

            let concurrency = resolve_concurrency();
            let nodes = topology.nodes.clone();
            let root_ref = &root;

            let resolved_nodes: HashMap<String, Topology> = stream::iter(nodes.into_iter())
                .map(|(name, node)| async move {
                    let node_t = topology::resolve(root_ref, &node, auth, sandbox, force).await;
                    (name, node_t)
                })
                .buffer_unordered(concurrency)
                .collect()
                .await;

            root.nodes = resolved_nodes;
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
    diff: bool,
) -> Topology {
    let mut root = topology::resolve_entity(topology, auth, sandbox, entity, diff).await;

    let concurrency = resolve_concurrency();
    let nodes = topology.nodes.clone();

    let resolved_nodes: HashMap<String, Topology> = stream::iter(nodes.into_iter())
        .map(|(name, node)| async move {
            let node_t = topology::resolve_entity(&node, auth, sandbox, entity, diff).await;
            (name, node_t)
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    root.nodes = resolved_nodes;
    root
}

async fn resolve_entity_component(
    auth: &Auth,
    sandbox: &str,
    topology: &Topology,
    entity: &Entity,
    component: &str,
) -> Topology {
    let mut root =
        topology::resolve_entity_component(topology, auth, sandbox, entity, component).await;

    let concurrency = resolve_concurrency();
    let nodes = topology.nodes.clone();

    let resolved_nodes: HashMap<String, Topology> = stream::iter(nodes.into_iter())
        .map(|(name, node)| async move {
            let node_t =
                topology::resolve_entity_component(&node, auth, sandbox, entity, component).await;
            (name, node_t)
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    root.nodes = resolved_nodes;
    root
}

pub async fn try_resolve(
    auth: &Auth,
    sandbox: &str,
    topology: &Topology,
    maybe_entity: &Option<String>,
    cache: bool,
    diff: bool,
) -> Topology {
    match maybe_entity {
        Some(e) => {
            let (entity, component) = Entity::as_entity_component(&e);
            if let Some(c) = component {
                resolve_entity_component(auth, sandbox, topology, &entity, &c).await
            } else {
                resolve_entity(auth, sandbox, topology, &entity, diff).await
            }
        }
        None => resolve(auth, sandbox, topology, cache, diff).await,
    }
}
