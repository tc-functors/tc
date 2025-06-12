use super::{Context, event, function, pool};
use authorizer::Auth;
use compiler::Topology;

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

pub async fn resolve_component(
    topology: &Topology,
    auth: &Auth,
    sandbox: &str,
    component: &str,
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

    match component {
        "events" => {
            partial_t.events = event::resolve(&ctx, &partial_t).await;
        }
        "functions" => {
            partial_t.functions = function::resolve(&ctx, &partial_t).await;
        }
        "layers" => {
            partial_t.functions = function::resolve(&ctx, &partial_t).await;
        }
        "pools" => {
            partial_t.pools = pool::resolve(&ctx, &partial_t).await;
        }

        _ => (),
    }
    partial_t
}
