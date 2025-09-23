mod channel;
mod event;
mod function;
pub mod guard;
mod mutation;
mod page;
mod pool;
mod queue;
mod resource;
mod role;
mod route;
mod schedule;
mod state;

use colored::Colorize;
use compiler::Entity;
use composer::{
    Function,
    Topology,
};
use kit::*;
use provider::Auth;
use std::{
    collections::HashMap,
    str::FromStr,
};
use tabled::{
    Style,
    Table,
};

pub async fn create(auth: &Auth, topology: &Topology, sync: bool) {
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
        roles,
        ..
    } = topology;

    println!(
        "Creating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    role::create_or_update(auth, roles, tags).await;
    function::create(auth, functions, &tags, sync).await;
    channel::create(&auth, channels).await;
    mutation::create(&auth, mutations, &tags).await;
    queue::create(&auth, queues).await;
    event::create(&auth, events, &tags).await;
    pool::create(&auth, pools).await;
    route::create(&auth, routes, &tags).await;
    let cfg = make_config(&auth, topology).await;
    page::create(&auth, pages, &cfg, sandbox).await;
    if let Some(f) = flow {
        state::create(&auth, &f, tags).await;
    }
}

async fn update_function(
    auth: &Auth,
    namespace: &str,
    sandbox: &str,
    f: &Function,
    tags: &HashMap<String, String>,
) {
    println!(
        "Updating function {}@{}.{}/functions/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &f.name
    );
    let mut fns: HashMap<String, Function> = HashMap::new();
    fns.insert(f.name.clone(), f.clone());
    function::update_code(auth, &fns, tags).await
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
        roles,
        ..
    } = topology;

    println!(
        "Updating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    role::create_or_update(&auth, roles, tags).await;
    function::update_code(&auth, functions, &tags).await;
    mutation::create(&auth, mutations, &tags).await;
    channel::create(&auth, channels).await;
    event::create(&auth, events, &tags).await;
    queue::create(&auth, queues).await;
    pool::create(&auth, pools).await;
    route::create(&auth, routes, &tags).await;
    let cfg = make_config(&auth, topology).await;
    page::create(&auth, pages, &cfg, &sandbox).await;
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
        Entity::Function => function::create(&auth, functions, tags, false).await,
        Entity::Mutation => mutation::create(&auth, mutations, tags).await,
        Entity::Queue => queue::create(&auth, queues).await,
        Entity::Channel => channel::create(&auth, channels).await,
        Entity::Schedule => schedule::create(&auth, schedules).await,
        Entity::Trigger => pool::create(&auth, pools).await,
        Entity::Route => route::create(&auth, routes, tags).await,
        Entity::Page => {
            let cfg = make_config(&auth, topology).await;
            page::create(&auth, pages, &cfg, &sandbox).await;
        }
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
        Entity::Function => function::update(&auth, functions, tags, component).await,
        Entity::Mutation => mutation::update(&auth, mutations, &component).await,
        Entity::Queue => queue::update(&auth, queues, component).await,
        Entity::Channel => channel::update(&auth, channels, component).await,
        Entity::Schedule => schedule::update(&auth, schedules).await,
        Entity::Trigger => pool::update(&auth, pools, component).await,
        Entity::Route => route::update(&auth, routes, component).await,
        Entity::Page => {
            let cfg = make_config(&auth, topology).await;
            page::update(&auth, pages, component, &cfg, &sandbox).await;
        }
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
        roles,
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
    role::delete(&auth, roles).await;
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
                Some(f) => {
                    update_function(
                        auth,
                        &topology.namespace,
                        &topology.sandbox,
                        &f,
                        &topology.tags,
                    )
                    .await
                }
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
    let Topology { fqn, .. } = topology;
    state::freeze(auth, fqn).await;
}

pub async fn unfreeze(auth: &Auth, topology: &Topology) {
    let Topology { fqn, .. } = topology;
    state::unfreeze(auth, fqn).await;
}

pub async fn try_list(auth: &Auth, topology: &Topology, maybe_entity: &Option<String>) {
    let Topology { functions, fqn, .. } = topology;
    match maybe_entity {
        Some(e) => {
            let entity = Entity::from_str(&e).unwrap();
            match entity {
                Entity::Function => {
                    let rs = function::list(auth, &functions).await;
                    let table = Table::new(rs).with(Style::psql()).to_string();
                    println!("{}", table);
                }
                Entity::Mutation => {
                    mutation::list(auth, &fqn).await;
                }
                Entity::State => {}
                _ => todo!(),
            }
        }
        None => {
            let rs = function::list(auth, &functions).await;
            let table = Table::new(rs).with(Style::psql()).to_string();
            println!("{}", table);
        }
    }
}

pub async fn list_all(auth: &Auth, sandbox: &str, format: &str) {
    let mut arns = resource::list(auth, sandbox).await;
    arns.sort();
    let grouped = resource::group_entities(arns.clone());
    match format {
        "json" => kit::pp_json(&grouped),
        _ => {
            for arn in &arns {
                println!("{}", &arn)
            }
            println!("");
            println!("{}", resource::count_of(&grouped));
        }
    }
}

pub async fn prune(auth: &Auth, sandbox: &str, filter: Option<String>) {
    let arns = resource::list(auth, sandbox).await;
    let arns = resource::filter_arns(arns, filter);
    let grouped = resource::group_entities(arns);
    println!("{}", resource::count_of(&grouped));
    let cont = guard::prompt("Do you want to delete these resources in given sandbox ?");
    if !cont {
        std::process::exit(1);
    }
    resource::delete_arns(auth, grouped).await;
}

pub async fn make_config(auth: &Auth, topology: &Topology) -> HashMap<String, String> {
    let Topology {
        fqn,
        sandbox,
        mutations,
        ..
    } = topology;
    let mut h: HashMap<String, String> = HashMap::new();
    if let Some(_m) = mutations.get("default") {
        let mutation_config = mutation::config(auth, fqn).await;
        h.extend(mutation_config);
    }
    h.insert(s!("REGION"), auth.region.clone());
    h.insert(s!("ENV"), auth.name.clone());
    h.insert(s!("SANDBOX"), sandbox.to_string());
    h
}
