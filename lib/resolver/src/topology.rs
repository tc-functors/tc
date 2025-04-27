use super::{
    Context,
    event,
    function,
    route,
};
use compiler::Topology;
use provider::Env;

pub async fn resolve(topology: &Topology, env: &Env, sandbox: &str) -> Topology {
    let ctx = Context {
        env: env.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
    };

    println!("Resolving topology {}", topology.namespace);
    let templated = topology.to_str();
    let rendered = ctx.render(&templated);
    let mut partial_t: Topology = serde_json::from_str(&rendered).unwrap();

    tracing::debug!("Resolving routes");
    partial_t.routes = route::resolve(&ctx, &partial_t).await;
    tracing::debug!("Resolving events");
    partial_t.events = event::resolve(&ctx, &partial_t).await;
    tracing::debug!("Resolving functions");
    partial_t.functions = function::resolve(&ctx, &partial_t).await;
    partial_t
}

pub async fn resolve_component(
    topology: &Topology,
    env: &Env,
    sandbox: &str,
    component: &str,
) -> Topology {
    let ctx = Context {
        env: env.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
    };

    println!("Resolving topology...");
    let templated = topology.to_str();
    let rendered = ctx.render(&templated);
    let mut partial_t: Topology = serde_json::from_str(&rendered).unwrap();

    println!("Resolving {}...", component);
    match component {
        "events" => {
            partial_t.events = event::resolve(&ctx, &partial_t).await;
        }
        "routes" => {
            partial_t.routes = route::resolve(&ctx, &partial_t).await;
        }
        "functions" => {
            partial_t.functions = function::resolve(&ctx, &partial_t).await;
        }
        "layers" => {
            partial_t.functions = function::resolve(&ctx, &partial_t).await;
        }

        _ => (),
    }
    partial_t
}
