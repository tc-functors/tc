use super::Context;
use compiler::Topology;
use super::{event, route, function};
use aws::Env;

pub async fn resolve(topology: &Topology, env: &Env, sandbox: &str) -> Topology {

    let ctx = Context {
        env: env.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
    };

    // resolve template
    let templated  = topology.to_str();
    let rendered = ctx.render(&templated);
    let mut partial_t: Topology = serde_json::from_str(&rendered).unwrap();

    // resolve by query

    partial_t.routes = route::resolve(&ctx, &partial_t).await;
    partial_t.events = event::resolve(&ctx, &partial_t).await;
    partial_t.functions = function::resolve(&ctx, &partial_t).await;

    partial_t
}
