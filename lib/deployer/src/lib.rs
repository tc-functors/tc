mod aws;
pub mod base;
pub mod channel;
pub mod event;
pub mod state;
pub mod function;
pub mod mutation;
pub mod pool;
pub mod queue;
pub mod role;
pub mod route;
pub mod schedule;

use authorizer::Auth;
use colored::Colorize;
use compiler::{
    Entity,
    Topology,
};

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
    event::create(&auth, events).await;
    pool::create(&auth, pools).await;
    route::create(&auth, routes).await;
    if let Some(f) = flow {
        state::create(&auth, &f, tags).await;
    }
}

async fn update(auth: &Auth, topology: &Topology) {
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
    event::create(&auth, events).await;
    queue::create(&auth, queues).await;
    pool::create(&auth, pools).await;
    route::create(&auth, routes).await;
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

        Entity::Event    => event::create(&auth, events).await,
        Entity::Function => function::create(&auth, functions).await,
        Entity::Mutation => mutation::create(&auth, mutations, tags).await,
        Entity::Queue    => queue::create(&auth, queues).await,
        Entity::Channel  => channel::create(&auth, channels).await,
        Entity::Schedule => schedule::create(&auth, schedules).await,
        Entity::Trigger  => pool::create(&auth, pools).await,
        Entity::Route    => route::create(&auth, routes).await,
        Entity::State    => {
            if let Some(f) = flow {
                state::create(&auth, f, tags).await;
            }
        }
    }

}

async fn update_component(
    auth: &Auth,
    topology: &Topology,
    entity: Entity,
    component: &str
) {
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

        Entity::Event    => event::update(&auth, events, component).await,
        Entity::Function => function::update(&auth, functions, component).await,
        Entity::Mutation => mutation::update(&auth, mutations, &component).await,
        Entity::Queue    => queue::update(&auth, queues, component).await,
        Entity::Channel  => channel::update(&auth, channels, component).await,
        Entity::Schedule => schedule::update(&auth, schedules).await,
        Entity::Trigger  => pool::update(&auth, pools, component).await,
        Entity::Route    => route::update(&auth, routes, component).await,
        Entity::State    => {
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

async fn delete_entity(
    auth: &Auth,
    topology: &Topology,
    entity: Entity
) {
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
        Entity::Event    => event::delete(&auth, events).await,
        Entity::Route    => route::delete(&auth, routes).await,
        Entity::Function => function::delete(&auth, functions).await,
        Entity::Mutation => mutation::delete(&auth, mutations).await,
        Entity::Schedule => schedule::delete(&auth, schedules).await,
        Entity::Trigger  => pool::delete(&auth, pools).await,
        Entity::Queue    => queue::delete(&auth, queues).await,
        Entity::Channel  => channel::delete(&auth, channels).await,
        Entity::State    => {
            if let Some(f) = flow {
                state::delete(&auth, f).await;
            }
        }
    }
}


async fn delete_component(
    auth: &Auth,
    topology: &Topology,
    entity: Entity,
    component: &str
) {
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

pub async fn try_update(
    auth: &Auth,
    topology: &Topology,
    maybe_entity: &Option<String>,
) {

    match maybe_entity {
        Some(e) => {
            let (entity, component) = Entity::as_entity_component(&e);
            match component {
                Some(c) => {
                    update_component(auth, topology, entity, &c).await
                },
                None => update_entity(auth, topology, entity).await
            }
        },
        None => update(auth, topology).await
    }
}


pub async fn try_delete(
    auth: &Auth,
    topology: &Topology,
    maybe_entity: &Option<String>,
) {

    match maybe_entity {
        Some(e) => {
            let (entity, component) = Entity::as_entity_component(&e);
            match component {
                Some(c) => {
                    delete_component(auth, topology, entity, &c).await
                },
                None => delete_entity(auth, topology, entity).await
            }
        },
        None => delete(auth, topology).await
    }
}
