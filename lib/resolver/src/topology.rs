use super::{
    Context,
    event,
    function,
    pool,
};
use authorizer::Auth;
use compiler::{
    Entity,
    Topology,
};

pub async fn resolve(topology: &Topology, auth: &Auth, sandbox: &str) -> Topology {
    let ctx = Context {
        auth: auth.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
        config: topology.config.to_owned(),
    };

    let templated = topology.to_str();
    let rendered = ctx.render(&templated);
    let mut partial_t: Topology = serde_json::from_str(&rendered).unwrap();

    tracing::debug!("Resolving events");
    partial_t.events = event::resolve(&ctx, &partial_t).await;
    tracing::debug!("Resolving pools");
    partial_t.pools = pool::resolve(&ctx, &partial_t).await;
    tracing::debug!("Resolving functions");
    partial_t.functions = function::resolve(&ctx, &partial_t).await;
    partial_t
}

pub async fn resolve_entity(
    topology: &Topology,
    auth: &Auth,
    sandbox: &str,
    entity: &Entity,
) -> Topology {
    let ctx = Context {
        auth: auth.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
        config: topology.config.to_owned(),
    };

    let templated = topology.to_str();
    let rendered = ctx.render(&templated);
    let mut partial_t: Topology = serde_json::from_str(&rendered).unwrap();

    match entity {
        Entity::Event => {
            partial_t.events = event::resolve(&ctx, &partial_t).await;
        }
        Entity::Function => {
            partial_t.functions = function::resolve(&ctx, &partial_t).await;
        }
        Entity::Trigger => {
            partial_t.pools = pool::resolve(&ctx, &partial_t).await;
        }
        _ => (),
    }
    partial_t
}
