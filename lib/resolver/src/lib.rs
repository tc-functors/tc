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

/// Per-loop-level cap on concurrent in-flight resolve futures.
/// Override via `TC_RESOLVE_CONCURRENCY=N`; values are clamped to
/// `[1, MAX_RESOLVE_CONCURRENCY]`.
///
/// **The cap applies independently at two loop levels** — the node
/// loop in [`resolve`] / [`resolve_entity`] / [`resolve_entity_component`]
/// and the function loop in [`function::resolve`] — so worst-case
/// in-flight resolutions is `cap × cap`. With the default of 16
/// that's 256 concurrent `resolve_runtime` calls. Higher caps offer
/// diminishing returns (the AWS SDK's default HTTP connection pool is
/// in the same ballpark) and risk tripping per-account throttles.
///
/// Per-call caches in [`function`] collapse `STS GetCallerIdentity`,
/// `Lambda ListLayerVersions`, and `APIGateway GetApis` long before
/// the wire, so the metric this cap is sized against is the
/// per-function-unique work — primarily `SSM GetParameter` (40 TPS
/// account limit). 256 in-flight × ~150 ms/call ≈ 1.7k requests/s,
/// comfortably within SSM's burst budget.
pub(crate) fn resolve_concurrency() -> usize {
    std::env::var("TC_RESOLVE_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .map(|n| n.min(MAX_RESOLVE_CONCURRENCY))
        .unwrap_or(16)
}

/// Hard upper bound on `TC_RESOLVE_CONCURRENCY`. Picked to stay
/// within the AWS SDK's typical HTTP connection-pool size and to
/// keep `cap × cap` worst-case in-flight futures bounded
/// (`64 × 64 = 4096`). Hitting this ceiling on a real topology
/// suggests either a configuration problem or a topology far beyond
/// what the resolver has been tested against — file an issue rather
/// than raising the cap.
pub(crate) const MAX_RESOLVE_CONCURRENCY: usize = 64;

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
