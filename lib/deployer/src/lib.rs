pub mod channel;
pub mod event;
pub mod flow;
pub mod function;
pub mod mutation;
pub mod queue;
pub mod role;
pub mod route;
pub mod schedule;
pub mod base;
pub mod pool;
mod aws;

use colored::Colorize;
use compiler::{
    Topology,
    spec::TopologyKind,
};
use authorizer::Auth;

pub fn maybe_component(c: Option<String>) -> String {
    match c {
        Some(comp) => comp,
        _ => "default".to_string(),
    }
}

fn prn_components() {
    let v: Vec<&str> = vec![
        "events",
        "functions",
        "layers",
        "roles",
        "routes",
        "flow",
        "vars",
        "logs",
        "mutations",
        "schedules",
        "queues",
        "channels",
        "base-roles",
        "pools"
    ];
    for x in v {
        println!("{x}");
    }
}

fn should_update_layers() -> bool {
    match std::env::var("LAYERS") {
        Ok(_) => true,
        Err(_e) => false,
    }
}

async fn create_flow(auth: &Auth, topology: &Topology) {
    let Topology {
        fqn,
        functions,
        routes,
        events,
        flow,
        mutations,
        queues,
        logs,
        tags,
        channels,
        ..
    } = topology;

    role::create_or_update(auth, &topology.roles()).await;
    function::create(auth, functions.clone()).await;
    if should_update_layers() {
        function::update_layers(auth, functions.clone()).await;
    }
    match flow {
        Some(f) => {
            flow::create(auth, tags, f.clone()).await;
            let sfn_arn = &auth.sfn_arn(&fqn);
            flow::enable_logs(auth, sfn_arn, logs.clone(), f).await;
            let role_name = "tc-base-api-role";
            let role_arn = &auth.role_arn(&role_name);
            route::create(&auth, role_arn, routes.clone()).await;
        }
        None => {
            let role_name = "tc-base-api-role";
            let role_arn = &auth.role_arn(&role_name);
            route::create(&auth, role_arn, routes.clone()).await;
        }
    }

    channel::create(&auth, channels).await;
    mutation::create(&auth, mutations, &tags).await;
    queue::create(&auth, queues).await;
    event::create(&auth, events).await;
}

async fn create_function(auth: &Auth, topology: &Topology) {
    let Topology {
        functions,
        routes,
        events,
        queues,
        mutations,
        tags,
        pools,
        channels,
        ..
    } = topology;
    role::create_or_update(&auth, &topology.roles()).await;
    function::create(&auth, functions.clone()).await;
    channel::create(&auth, channels).await;
    mutation::create(&auth, mutations, tags).await;
    queue::create(&auth, queues).await;
    event::create(&auth, events).await;
    pool::create(&auth, pools.clone()).await;
    function::update_concurrency(&auth, functions.clone()).await;

    let role_name = "tc-base-api-role";
    let role_arn = &auth.role_arn(&role_name);
    route::create(&auth, role_arn, routes.clone()).await;
}

pub async fn create(auth: &Auth, topology: &Topology) {
    let Topology {
        kind,
        namespace,
        version,
        sandbox,
        ..
    } = topology;

    println!(
        "Creating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    match kind {
        TopologyKind::StepFunction => create_flow(auth, &topology).await,
        _ => create_function(auth, &topology).await,
    }
}

pub async fn update(auth: &Auth, topology: &Topology) {
    let Topology {
        namespace,
        version,
        functions,
        flow,
        mutations,
        sandbox,
        events,
        queues,
        tags,
        ..
    } = topology;

    println!(
        "Updating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    function::update_code(&auth, functions.clone()).await;
    function::update_concurrency(&auth, functions.clone()).await;
    match flow {
        Some(f) => flow::create(&auth, tags, f.clone()).await,
        None => (),
    }
    mutation::create(&auth, &mutations, &tags).await;
    event::create(&auth, &events).await;
    queue::create(&auth, &queues).await;
}

pub async fn update_component(auth: &Auth, topology: &Topology, component: Option<String>) {
    let component = maybe_component(component);
    let Topology {
        fqn,
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
        logs,
        pools,
        ..
    } = topology.clone();

    println!(
        "Updating functor {}@{}.{}/{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version,
        &component
    );

    match component.as_str() {
        "events" => {
            event::create(&auth, &events).await;
        }

        "functions" => {
            function::create(&auth, functions).await;
        }

        "layers" => {
            function::update_layers(&auth, functions).await;
        }

        "routes" => match flow {
            Some(f) => {
                //let role_name = "tc-base-api-role";
                let role_arn = &auth.role_arn(&f.role.name);
                route::create(&auth, role_arn, routes).await;
            }
            None => {
                let role_name = "tc-base-api-role";
                let role_arn = &auth.role_arn(&role_name);
                route::create(&auth, role_arn, routes).await;
            }
        },

        "runtime" => {
            function::update_runtime_version(&auth, functions).await;
        }

        "vars" => {
            function::update_vars(&auth, functions).await;
        }

        "concurrency" => {
            function::update_concurrency(&auth, functions).await;
        }

        "tags" => {
            function::update_tags(&auth, functions).await;
            match flow {
                Some(f) => flow::update_tags(&auth, &f.name, tags).await,
                None => println!("No flow defined, skipping"),
            }
        }
        "flow" => match flow {
            Some(f) => flow::update_definition(&auth, &tags, f).await,
            None => println!("No flow defined, skipping"),
        },

        "mutations" => mutation::create(&auth, &mutations, &tags).await,

        "roles" => role::create_or_update(&auth, &topology.roles()).await,

        "schedules" => schedule::create(&auth, &namespace, schedules).await,

        "queues" => queue::create(&auth, &queues).await,

        "channels" => channel::create(&auth, &channels).await,

        "base-roles" => base::create_roles(&auth).await,

        "logs" => match flow {
            Some(f) => {
                let sfn_arn = auth.sfn_arn(&fqn);
                flow::enable_logs(&auth, &sfn_arn, logs.clone(), &f).await;
            }
            None => (),
        },

        "pools" => pool::create(&auth, pools).await,

        "all" => {
            role::create_or_update(&auth, &topology.roles()).await;
            function::create(&auth, functions.clone()).await;
            match flow {
                Some(f) => flow::create(&auth, &tags, f).await,
                None => (),
            }
            function::update_vars(&auth, functions.clone()).await;
            function::update_tags(&auth, functions).await;
        }

        _ => {
            if kit::file_exists(&component) {
                let c = kit::strip(&component, "/").replace("_", "-");
                match functions.get(&c) {
                    Some(f) => {
                        builder::build(&f, None, None, None, None).await;
                        let p = auth.name.to_string();
                        let role = auth.assume_role.to_owned();
                        function::create_function(p, role, f.clone()).await;
                    }
                    None => panic!("No valid function found"),
                }
            } else {
                println!("Available components: ");
                prn_components();
            }
        }
    }
}

pub async fn delete(auth: &Auth, topology: &Topology) {
    let Topology {
        fqn,
        namespace,
        functions,
        flow,
        sandbox,
        mutations,
        routes,
        version,
        queues,
        ..
    } = topology.clone();

    println!(
        "Deleting functor: {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version
    );

    match flow {
        Some(f) => {
            let sfn_arn = auth.sfn_arn(&fqn);
            flow::disable_logs(&auth, &sfn_arn).await;
            flow::delete(&auth, f.clone()).await;
        }
        None => println!("No flow defined, skipping"),
    }
    function::delete(&auth, functions).await;
    role::delete(&auth, &topology.roles()).await;
    route::delete(&auth, "", routes).await;

    mutation::delete(&auth, &mutations).await;
    queue::delete(&auth, &queues).await;
}

pub async fn delete_component(auth: &Auth, topology: Topology, component: Option<String>) {
    let component = maybe_component(component);
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
        ..
    } = topology;

    println!(
        "Deleting functor: {}@{}.{}/{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &auth.name.blue(),
        &version,
        &component
    );

    match component.as_str() {
        "events" => event::delete(&auth, &events).await,
        "schedules" => schedule::delete(&auth, &namespace, schedules).await,
        "routes" => route::delete(&auth, "", routes).await,
        "functions" => function::delete(&auth, functions).await,
        "mutations" => mutation::delete(&auth, &mutations).await,
        "pools" => pool::delete(&auth, pools).await,
        "flow" => match flow {
            Some(f) => flow::delete(&auth, f).await,
            None => (),
        },
        _ => prn_components(),
    }
}
