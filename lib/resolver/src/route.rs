use compiler::Route;
use super::{Context, Topology};
use std::collections::HashMap;

fn is_stable(sandbox: &str) -> bool {
    sandbox == "stable"
}

pub async fn resolve(context: &Context, topology: &Topology) -> HashMap<String, Route> {
    let mut routes: HashMap<String, Route> = HashMap::new();

    let sandbox = &context.sandbox;

    for (id, route) in &topology.routes {
        let path = if is_stable(&sandbox) {
            route.path.to_owned()
        } else {
            format!("/{}{}", sandbox, &route.path)
        };
        let mut r: Route = route.clone();

        r.path = path;
        routes.insert(id.to_string(), r);
    }

    routes
}
