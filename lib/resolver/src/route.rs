use super::{
    Context,
    Topology,
};
use compiler::Route;
use std::collections::HashMap;

fn is_stable(sandbox: &str) -> bool {
    sandbox == "stable"
}

pub async fn resolve(context: &Context, topology: &Topology) -> HashMap<String, Route> {
    let mut routes: HashMap<String, Route> = HashMap::new();

    let sandbox = &context.sandbox;

    for (id, route) in &topology.routes {
        let gateway = if is_stable(&sandbox) {
            route.gateway.to_owned()
        } else {
            format!("{}_{}", &route.gateway, sandbox)
        };
        let mut r: Route = route.clone();

        r.gateway = gateway;
        routes.insert(id.to_string(), r);
    }

    routes
}
