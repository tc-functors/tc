mod aws;
pub mod base;
pub mod channel;
pub mod event;
pub mod function;
pub mod mutation;
pub mod page;
pub mod pool;
pub mod queue;
pub mod role;
pub mod route;
pub mod schedule;
pub mod state;

use authorizer::Auth;
use colored::Colorize;
use composer::{
    Entity,
    Function,
    Topology,
};
use std::collections::HashMap;
use tabled::{Style, Table};
use std::str::FromStr;

pub async fn create(auth: &Auth, topology: &Topology) {
    let Topology {
        namespace,
        version,
        sandbox,
        functions,
        routes,
        events,
        queues,
        mutations,
        tags,
        pools,
        channels,
        pages,
        flow,
        ..
    } = topology;

    println!(
        "Creating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    role::create_or_update(auth, &topology.roles()).await;
    function::create(auth, functions).await;
    channel::create(&auth, channels).await;
    mutation::create(&auth, mutations, &tags).await;
    queue::create(&auth, queues).await;
    event::create(&auth, events, &tags).await;
    pool::create(&auth, pools).await;
    route::create(&auth, routes).await;
    page::create(&auth, pages).await;
    if let Some(f) = flow {
        state::create(&auth, &f, tags).await;
    }
}

async fn update_function(auth: &Auth, namespace: &str, sandbox: &str, f: &Function) {
    println!(
        "Updating function {}@{}.{}/functions/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &f.name
    );
    let mut fns: HashMap<String, Function> = HashMap::new();
    fns.insert(f.name.clone(), f.clone());
    function::update_code(auth, &fns).await
}

async fn update_topology(auth: &Auth, topology: &Topology) {
    let Topology {
        namespace,
        version,
        functions,
        flow,
        mutations,
        channels,
        sandbox,
        events,
        queues,
        tags,
        pools,
        routes,
        pages,
        ..
    } = topology;

    println!(
        "Updating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    function::update_code(&auth, functions).await;
    mutation::create(&auth, mutations, &tags).await;
    channel::create(&auth, channels).await;
    event::create(&auth, events, &tags).await;
    queue::create(&auth, queues).await;
    pool::create(&auth, pools).await;
    route::create(&auth, routes).await;
    page::create(&auth, pages).await;
    if let Some(f) = flow {
        state::create(&auth, &f, tags).await;
    }
}

async fn update_entity(auth: &Auth, topology: &Topology, entity: Entity) {
    let Topology {
        version,
        namespace,
        sandbox,
        functions,
        events,
        routes,
        flow,
        mutations,
        schedules,
        queues,
        tags,
        channels,
        pools,
        pages,
        ..
    } = topology;

    println!(
        "Updating functor {}@{}.{}/{}/{}",
        namespace.green(),
        sandbox.cyan(),
        &auth.name.blue(),
        version,
        &entity.to_str()
    );
    match entity {
        Entity::Event => event::create(&auth, events, tags).await,
        Entity::Function => function::create(&auth, functions).await,
        Entity::Mutation => mutation::create(&auth, mutations, tags).await,
        Entity::Queue => queue::create(&auth, queues).await,
        Entity::Channel => channel::create(&auth, channels).await,
        Entity::Schedule => schedule::create(&auth, schedules).await,
        Entity::Trigger => pool::create(&auth, pools).await,
        Entity::Route => route::create(&auth, routes).await,
        Entity::Page => page::create(&auth, pages).await,
        Entity::State => {
            if let Some(f) = flow {
                state::create(&auth, f, tags).await;
            }
        }
    }
}

async fn update_component(auth: &Auth, topology: &Topology, entity: Entity, component: &str) {
    let Topology {
        version,
        namespace,
        sandbox,
        functions,
        events,
        routes,
        flow,
        mutations,
        schedules,
        queues,
        tags,
        channels,
        pools,
        pages,
        ..
    } = topology;

    println!(
        "Updating functor {}@{}.{}/{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version,
        &entity.to_str()
    );

    match entity {
        Entity::Event => event::update(&auth, events, tags, component).await,
        Entity::Function => function::update(&auth, functions, component).await,
        Entity::Mutation => mutation::update(&auth, mutations, &component).await,
        Entity::Queue => queue::update(&auth, queues, component).await,
        Entity::Channel => channel::update(&auth, channels, component).await,
        Entity::Schedule => schedule::update(&auth, schedules).await,
        Entity::Trigger => pool::update(&auth, pools, component).await,
        Entity::Route => route::update(&auth, routes, component).await,
        Entity::Page => page::update(&auth, pages, component).await,
        Entity::State => {
            if let Some(f) = flow {
                state::update(&auth, f, tags, component).await;
            }
        }
    }
}

async fn delete(auth: &Auth, topology: &Topology) {
    let Topology {
        sandbox,
        namespace,
        functions,
        flow,
        mutations,
        routes,
        version,
        queues,
        ..
    } = topology;

    println!(
        "Deleting functor: {}@{}.{}/{}",
        namespace.green(),
        sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    if let Some(f) = flow {
        state::delete(auth, f).await;
    }
    function::delete(&auth, functions).await;
    role::delete(&auth, &topology.roles()).await;
    route::delete(&auth, routes).await;
    mutation::delete(&auth, mutations).await;
    queue::delete(&auth, queues).await;
}

async fn delete_entity(auth: &Auth, topology: &Topology, entity: Entity) {
    let Topology {
        namespace,
        functions,
        events,
        routes,
        mutations,
        schedules,
        flow,
        sandbox,
        version,
        pools,
        queues,
        channels,
        pages,
        ..
    } = topology;

    println!(
        "Deleting functor: {}@{}.{}/{}/{}",
        &namespace.red(),
        &sandbox.red(),
        &auth.name.blue(),
        &version,
        entity.to_str()
    );

    match entity {
        Entity::Event => event::delete(&auth, events).await,
        Entity::Route => route::delete(&auth, routes).await,
        Entity::Function => function::delete(&auth, functions).await,
        Entity::Mutation => mutation::delete(&auth, mutations).await,
        Entity::Schedule => schedule::delete(&auth, schedules).await,
        Entity::Trigger => pool::delete(&auth, pools).await,
        Entity::Queue => queue::delete(&auth, queues).await,
        Entity::Channel => channel::delete(&auth, channels).await,
        Entity::Page => page::delete(&auth, pages).await,
        Entity::State => {
            if let Some(f) = flow {
                state::delete(&auth, f).await;
            }
        }
    }
}

async fn delete_component(auth: &Auth, topology: &Topology, entity: Entity, component: &str) {
    let Topology {
        namespace,
        sandbox,
        version,
        ..
    } = topology;

    println!(
        "Deleting functor: {}@{}.{}/{}/{}/{}",
        namespace.green(),
        sandbox.cyan(),
        &auth.name.blue(),
        version,
        entity.to_str(),
        &component
    );
}

// pub interfaces

pub async fn try_update(auth: &Auth, topology: &Topology, maybe_entity: &Option<String>) {
    match maybe_entity {
        Some(e) => {
            let (entity, component) = Entity::as_entity_component(&e);
            match component {
                Some(c) => update_component(auth, topology, entity, &c).await,
                None => update_entity(auth, topology, entity).await,
            }
        }
        None => {
            let dir = kit::pwd();
            let maybe_function = topology.current_function(&dir);
            match maybe_function {
                Some(f) => update_function(auth, &topology.namespace, &topology.sandbox, &f).await,
                None => update_topology(auth, topology).await,
            }
        }
    }
}

pub async fn try_delete(auth: &Auth, topology: &Topology, maybe_entity: &Option<String>) {
    match maybe_entity {
        Some(e) => {
            let (entity, component) = Entity::as_entity_component(&e);
            match component {
                Some(c) => delete_component(auth, topology, entity, &c).await,
                None => delete_entity(auth, topology, entity).await,
            }
        }
        None => delete(auth, topology).await,
    }
}

pub async fn freeze(auth: &Auth, topology: &Topology) {
    let Topology { fqn, .. }  = topology;
    state::freeze(auth, fqn).await;

}

pub async fn unfreeze(auth: &Auth, topology: &Topology) {
    let Topology { fqn, .. }  = topology;
    state::unfreeze(auth, fqn).await;
}

pub async fn try_list(auth: &Auth, topology: &Topology, maybe_entity: &Option<String>) {
    let Topology { functions, fqn, .. }  = topology;
    match maybe_entity {
        Some(e) => {
            let entity = Entity::from_str(&e).unwrap();
            match entity {
                Entity::Function => {
                    let rs = function::list(auth, &functions).await;
                    let table = Table::new(rs).with(Style::psql()).to_string();
                    println!("{}", table);
                },
                Entity::Mutation => {
                    mutation::list(auth, &fqn).await;
                },
                Entity::State => {

                }
                _ => todo!()
            }
        },
        None => {
            let rs = function::list(auth, &functions).await;
            let table = Table::new(rs).with(Style::psql()).to_string();
            println!("{}", table);
        }
    }
}

// guards
pub fn should_abort(sandbox: &str) -> bool {
    let yes = match std::env::var("CIRCLECI") {
        Ok(_) => false,
        Err(_) => match std::env::var("TC_FORCE_DEPLOY") {
            Ok(_) => false,
            Err(_) => true,
        },
    };
    yes && (sandbox == "stable")
}

pub fn guard(sandbox: &str) {
    if should_abort(sandbox) {
        std::panic::set_hook(Box::new(|_| {
            println!("Cannot create stable sandbox outside CI");
        }));
        panic!("Cannot create stable sandbox outside CI")
    }
}
