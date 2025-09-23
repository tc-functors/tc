mod function;
mod state;

use compiler::{Entity, BuildKind};
use composer::{
    Topology,
};
use kit as u;
use provider::aws::Auth;

pub async fn emulate(auth: &Auth, topology: &Topology, entity_component: &str, shell: bool) {
    let (entity, component) = Entity::as_entity_component(entity_component);
    match entity {
        Entity::Function => {
            let dir = u::pwd();
            let maybe_function = match component {
                Some(c) => topology.functions.get(&c).cloned(),
                None => {
                    if let Some(f) = composer::current_function(&dir) {
                        topology.functions.get(&f.name).cloned()
                    } else {
                        None
                    }
                }
            };
            if let Some(func) = maybe_function {
                let kind = &func.build.kind;
                match kind {
                    BuildKind::Image => function::image::run(auth, &dir, &func, shell).await,
                    _ => function::default::run(auth, &dir, &func).await,
                }
            }
        }
        Entity::State => state::run(&auth).await,
        _ => todo!(),
    }
}
