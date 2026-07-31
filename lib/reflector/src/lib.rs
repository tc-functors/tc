use compiler::Entity;
use composer::Topology;
mod mutation;
use provider::Auth;
use std::str::FromStr;

async fn reflect_entity(auth: &Auth, fqn: &str, entity: Entity) {
    match entity {
        Entity::Mutation => mutation::introspect(auth, fqn).await,
        _ => (),
    }
}

async fn reflect_component(auth: &Auth, component: &str) {
    println!("{} {}", auth.name, component);
}

pub async fn reflect(
    auth: &Auth,
    topology: &Topology,
    sandbox: &str,
    maybe_entity: Option<String>,
) {
    let namespace = &topology.namespace;
    let fqn = format!("{}_{}", namespace, sandbox);

    if let Some(e) = maybe_entity {
        match Entity::from_str(&e) {
            Ok(entity) => reflect_entity(auth, &fqn, entity).await,
            Err(_) => reflect_component(auth, &e).await,
        }
    } else {
        println!("Reflecting topology")
    }
}
