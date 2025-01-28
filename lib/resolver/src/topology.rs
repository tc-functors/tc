use super::Context;
use compiler::Topology;
use super::{event, route, function, cache};
use aws::Env;

fn make_cache_key(namespace: &str, profile: &str, sandbox: &str) -> String {
    format!("{}-{}-{}", namespace, profile, sandbox)
}

async fn write_cache(key: &str, t: &Topology) {
    let s = serde_json::to_string(t).unwrap();
    cache::write(key, &s).await
}

async fn read_cache(key: &str) -> Option<Topology> {
    if cache::has_key(key) {
        let s = cache::read(key);
        let t: Topology = serde_json::from_str(&s).unwrap();
        Some(t)
    } else {
        None
    }
}


async fn do_resolve(topology: &Topology, env: &Env, sandbox: &str) -> Topology {

    let ctx = Context {
        env: env.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
    };

    println!("Rendering templated topology...");
    let templated  = topology.to_str();
    let rendered = ctx.render(&templated);
    let mut partial_t: Topology = serde_json::from_str(&rendered).unwrap();


    // resolve by query
    println!("Resolving routes({})...", &partial_t.routes.len());
    partial_t.routes = route::resolve(&ctx, &partial_t).await;

    println!("Resolving events({})...", &partial_t.events.len());
    partial_t.events = event::resolve(&ctx, &partial_t).await;

    println!("Resolving functions({})...", &partial_t.functions.len());
    partial_t.functions = function::resolve(&ctx, &partial_t).await;


    partial_t
}

pub async fn resolve(topology: &Topology, env: &Env, sandbox: &str, cache: bool) -> Topology {
    if cache {
        let key = make_cache_key(&topology.namespace, &env.name, sandbox);
        let t = read_cache(&key).await;
        match t {
            Some(topo) => topo,
            None => {
                let topo = do_resolve(topology, env, sandbox).await;
                write_cache(&key, &topo).await;
                topo
            }
        }
    } else {
        do_resolve(topology, env, sandbox).await
    }
}

pub async fn resolve_component(topology: &Topology, env: &Env, sandbox: &str, component: &str) -> Topology {

    let ctx = Context {
        env: env.clone(),
        namespace: topology.namespace.to_owned(),
        sandbox: sandbox.to_string(),
        trace: true,
    };

    println!("Rendering templated topology...");
    let templated  = topology.to_str();
    let rendered = ctx.render(&templated);
    let mut partial_t: Topology = serde_json::from_str(&rendered).unwrap();

    println!("Resolving {}...", component);
    match component {
        "events" => {
            partial_t.events = event::resolve(&ctx, &partial_t).await;
        },
        "routes" => {
            partial_t.routes = route::resolve(&ctx, &partial_t).await;
        },
        "functions" => {
            partial_t.functions = function::resolve(&ctx, &partial_t).await;
        }
        _ => ()

    }
    partial_t
}
