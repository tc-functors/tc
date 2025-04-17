pub mod event;
pub mod flow;
pub mod function;
pub mod mutation;
pub mod queue;
pub mod role;
pub mod route;
pub mod schedule;
pub mod channel;

use colored::Colorize;
use compiler::Topology;
use compiler::spec::TopologyKind;
use aws::Env;

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
        "channels"
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

async fn create_flow(env: &Env, topology: &Topology) {
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
        ..
    } = topology;

    role::create_or_update(&env, &topology.roles()).await;
    function::create(env, functions.clone()).await;
    if should_update_layers() {
        function::update_layers(env, functions.clone()).await;
    }
    match flow {
        Some(f) => {
            flow::create(env, tags, f.clone()).await;
            let sfn_arn = &env.sfn_arn(&fqn);
            flow::enable_logs(&env, sfn_arn, logs.clone(), f).await;
            //route::create(&env, sfn_arn, &f.default_role, routes.clone()).await;
        }
        None => {
            let role_name = "tc-base-api-role";
            let role_arn = &env.role_arn(&role_name);
            route::create(&env, role_arn, routes.clone()).await;
        }
    }

    mutation::create(&env, mutations, &tags).await;
    queue::create(&env, queues).await;
    event::create(&env, events).await;
}

async fn create_function(env: &Env, topology: &Topology) {
    let Topology {
        functions,
        routes,
        events,
        queues,
        mutations,
        tags,
        ..
    } = topology;
    role::create_or_update(&env, &topology.roles()).await;
    function::create(&env, functions.clone()).await;
    mutation::create(&env, mutations, tags).await;
    queue::create(&env, queues).await;
    event::create(&env, events).await;
    function::update_concurrency(&env, functions.clone()).await;

    let role_name = "tc-base-api-role";
    let role_arn = &env.role_arn(&role_name);
    route::create(&env, role_arn, routes.clone()).await;
}

pub async fn create(env: &Env, topology: &Topology) {
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
        &env.name.blue(),
        &version
    );

     match kind {
        TopologyKind::StepFunction => create_flow(env, &topology).await,
        _  => create_function(env, &topology).await,
    }
}

pub async fn update(env: &Env, topology: &Topology) {
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
        &env.name.blue(),
        &version
    );

    function::update_code(&env, functions.clone()).await;
    function::update_concurrency(&env, functions.clone()).await;
    match flow {
        Some(f) => flow::create(&env, tags, f.clone()).await,
        None => (),
    }
    mutation::create(&env, &mutations, &tags).await;
    event::create(&env, &events).await;
    queue::create(&env, &queues).await;
}

pub async fn update_component(env: &Env, topology: &Topology, component: Option<String>) {
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
        ..
    } = topology.clone();


    println!(
        "Updating functor {}@{}.{}/{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &env.name.blue(),
        &version,
        &component
    );

    match component.as_str() {
        "events" => {
            event::create(&env, &events).await;
        }

        "functions" => {
            function::create(&env, functions).await;
        }

        "layers" => {
            function::update_layers(&env, functions).await;
        }

        "routes" => match flow {
            Some(f) => {
                route::create(&env, &f.role.name, routes).await;
            }
            None => {
                let role_name = "tc-base-api-role";
                let role_arn = &env.role_arn(&role_name);
                route::create(&env, role_arn, routes).await;
            }
        },

        "runtime" => {
            function::update_runtime_version(&env, functions).await;
        }

        "vars" => {
            function::update_vars(&env, functions).await;
        }

        "concurrency" => {
            function::update_concurrency(&env, functions).await;
        }

        "tags" => {
            function::update_tags(&env, functions).await;
            match flow {
                Some(f) => flow::update_tags(&env, &f.name, tags).await,
                None => println!("No flow defined, skipping"),
            }
        }
        "flow" => match flow {
            Some(f) => flow::update_definition(&env, &tags, f).await,
            None => println!("No flow defined, skipping"),
        },

        "mutations" => mutation::create(&env, &mutations, &tags).await,

        "roles" => role::create_or_update(&env, &topology.roles()).await,

        "schedules" => schedule::create(&env, &namespace, schedules).await,

        "queues" => queue::create(&env, &queues).await,

        "channels" => channel::create(&env, &channels).await,

        "logs" => match flow {
            Some(f) => {
                let sfn_arn = env.sfn_arn(&fqn);
                flow::enable_logs(&env, &sfn_arn, logs.clone(), &f).await;
            },
            None => ()
        },

        "all" => {
            role::create_or_update(&env, &topology.roles()).await;
            function::create(&env, functions.clone()).await;
            match flow {
                Some(f) => flow::create(&env, &tags, f).await,
                None => (),
            }
            function::update_vars(&env, functions.clone()).await;
            function::update_tags(&env, functions).await;
        }

        _ => {
            if kit::file_exists(&component) {
                let c = kit::strip(&component, "/").replace("_", "-");
                match functions.get(&c) {
                    Some(f) => {
                        builder::build(&f.dir, None, None).await;
                        let p = env.name.to_string();
                        let role = env.assume_role.to_owned();
                        let config = env.config.to_owned();
                        function::create_function(p, role, config, f.clone()).await;
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


pub async fn delete(env: &Env, topology: &Topology) {
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
        &env.name.blue(),
        &version
    );

    match flow {
        Some(f) => {
            let sfn_arn = env.sfn_arn(&fqn);
            flow::disable_logs(&env, &sfn_arn).await;
            flow::delete(&env, f.clone()).await;
        }
        None => println!("No flow defined, skipping"),
    }
    function::delete(&env, functions).await;
    role::delete(&env, &topology.roles()).await;
    route::delete(&env, "", routes).await;

    mutation::delete(&env, &mutations).await;
    queue::delete(&env, &queues).await;
}

pub async fn delete_component(env: &Env, topology: Topology, component: Option<String>) {
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
        ..
    } = topology;

    println!(
        "Deleting functor: {}@{}.{}/{}/{}",
        &namespace.green(),
        &sandbox.cyan(),
        &env.name.blue(),
        &version,
        &component
    );

    match component.as_str() {
        "events" => event::delete(&env, &events).await,
        "schedules" => schedule::delete(&env, &namespace, schedules).await,
        "routes" => route::delete(&env, "", routes).await,
        "functions" => function::delete(&env, functions).await,
        "mutations" => mutation::delete(&env, &mutations).await,
        "flow" => match flow {
            Some(f) => flow::delete(&env, f).await,
            None => (),
        },
        _ => prn_components()
    }
}
