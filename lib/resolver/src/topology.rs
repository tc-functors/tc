use super::{
    Context,
    event,
    function,
    pool,
};
use authorizer::Auth;
use composer::{
    Entity,
    Topology,
};
use function::Root;

pub async fn resolve(
    root: &Topology,
    topology: &Topology,
    auth: &Auth,
    sandbox: &str,
    diff: bool,
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

    let Topology {
        namespace,
        fqn,
        version,
        kind,
        ..
    } = root;

    let rt = Root {
        namespace: namespace.to_string(),
        fqn: ctx.render(&fqn),
        version: version.to_string(),
        kind: kind.clone(),
    };

    tracing::debug!("Resolving events {}", topology.namespace);
    partial_t.events = event::resolve(&ctx, &partial_t).await;
    tracing::debug!("Resolving pools {}", topology.namespace);
    partial_t.pools = pool::resolve(&ctx, &partial_t).await;
    tracing::debug!("Resolving functions {}", topology.namespace);

    partial_t.functions = function::resolve(&ctx, &rt, &partial_t, diff).await;
    partial_t
}

pub async fn resolve_entity(
    topology: &Topology,
    auth: &Auth,
    sandbox: &str,
    entity: &Entity,
    diff: bool,
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

    let Topology {
        namespace,
        fqn,
        version,
        kind,
        ..
    } = topology;

    let rt = Root {
        namespace: namespace.to_string(),
        fqn: ctx.render(&fqn),
        version: version.to_string(),
        kind: kind.clone(),
    };

    match entity {
        Entity::Event => {
            partial_t.events = event::resolve(&ctx, &partial_t).await;
        }
        Entity::Function => {
            partial_t.functions = function::resolve(&ctx, &rt, &partial_t, diff).await;
        }
        Entity::Trigger => {
            partial_t.pools = pool::resolve(&ctx, &partial_t).await;
        }
        _ => (),
    }
    partial_t
}

pub async fn resolve_entity_component(
    topology: &Topology,
    auth: &Auth,
    sandbox: &str,
    entity: &Entity,
    component: &str,
) -> Topology {
    tracing::debug!("Resolving {}", component);

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

    let Topology {
        namespace,
        fqn,
        version,
        kind,
        ..
    } = topology;

    let rt = Root {
        namespace: namespace.to_string(),
        fqn: ctx.render(&fqn),
        version: version.to_string(),
        kind: kind.clone(),
    };

    match entity {
        Entity::Event => {
            partial_t.events = event::resolve(&ctx, &partial_t).await;
        }
        Entity::Function => {
            partial_t.functions = function::resolve_given(&ctx, &rt, &partial_t, component).await;
        }
        Entity::Trigger => {
            partial_t.pools = pool::resolve(&ctx, &partial_t).await;
        }
        _ => (),
    }
    partial_t
}
