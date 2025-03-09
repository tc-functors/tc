pub mod event;
pub mod flow;
pub mod function;
pub mod mutation;
pub mod queue;
pub mod role;
pub mod route;
pub mod schedule;

use colored::Colorize;
use compiler::Topology;
use compiler::spec::TopologyKind;
use aws::Env;

fn maybe_component(c: Option<String>) -> String {
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
        "mutations",
        "schedules",
        "queues",
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
        functions,
        routes,
        events,
        flow,
        mutations,
        queues,
        ..
    } = topology;

    role::create(&env, &topology.roles()).await;
    function::create(env, functions.clone()).await;
    if should_update_layers() {
        function::update_layers(env, functions.clone()).await;
    }
    match flow {
        Some(f) => {
            flow::create(env, f.clone()).await;
            //flow::enable_logs(&env, sfn_arn, logs.clone()).await;
            //route::create(&env, sfn_arn, &f.default_role, routes.clone()).await;
        }
        None => {
            let role_name = "tc-base-api-role";
            let role_arn = &env.role_arn(&role_name);
            route::create(&env, role_arn, routes.clone()).await;
        }
    }

    mutation::create(&env, mutations).await;
    queue::create(&env, queues).await;
    event::create(&env, events).await;
}

async fn create_function(env: &Env, topology: &Topology) {
    let Topology {
        functions,
        ..
    } = topology;
    role::create(&env, &topology.roles()).await;
    function::create(&env, functions.clone()).await;
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
        TopologyKind::Function => create_function(env, &topology).await,
        TopologyKind::Evented => ()
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
    match flow {
        Some(f) => flow::create(&env, f.clone()).await,
        None => (),
    }
    mutation::create(&env, &mutations).await;
    event::create(&env, &events).await;
    queue::create(&env, &queues).await;
}

pub async fn update_component(env: &Env, topology: &Topology, component: Option<String>) {
    let component = maybe_component(component);
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

        "tags" => {
            function::update_tags(&env, functions).await;
            match flow {
                Some(f) => flow::update_tags(&env, &f.name, f.tags).await,
                None => println!("No flow defined, skipping"),
            }
        }
        "flow" => match flow {
            Some(f) => flow::create(&env, f).await,
            None => println!("No flow defined, skipping"),
        },

        "mutations" => mutation::create(&env, &mutations).await,

        "schedules" => schedule::create(&env, &namespace, schedules).await,

        "queues" => queue::create(&env, &queues).await,

        "all" => {
            role::create(&env, &topology.roles()).await;
            function::create(&env, functions.clone()).await;
            match flow {
                Some(f) => flow::create(&env, f).await,
                None => (),
            }
            function::update_vars(&env, functions.clone()).await;
            function::update_tags(&env, functions).await;
        }

        _ => prn_components()
    }
}


pub async fn delete(env: &Env, topology: &Topology) {
    let Topology {
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
            //let sfn_name = f.clone().name;
            //let sfn_arn = env.sfn_arn(&sfn_name);
            //flow::disable_logs(&env, &sfn_arn).await;
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
