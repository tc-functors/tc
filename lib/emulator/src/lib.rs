mod function;
mod state;
mod aws;

use authorizer::Auth;
use composer::{Entity, Topology, BuildKind};
use kit as u;

pub async fn emulate(auth: &Auth, topology: &Topology, entity_component: &str) {
    let (entity, component) = Entity::as_entity_component(entity_component);
    match entity {
        Entity::Function => {
            let dir = u::pwd();
            let maybe_function = match component {
                Some(c) => topology.functions.get(&c).cloned(),
                None => composer::current_function(&dir)
            };
            if let Some(func) = maybe_function {
                let kind = &func.build.kind;
                match kind {
                    BuildKind::Image => function::image::run(auth, &dir, &func).await,
                    _ => function::default::run(auth, &dir, &func).await
                }
            }
        }
        Entity::State => state::run(&auth).await,
        _ => todo!(),
    }
}
