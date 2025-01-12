pub mod event;
pub mod flow;
pub mod function;
pub mod mutation;
pub mod queue;
pub mod role;
pub mod route;
pub mod schedule;

use resolver::plan::Plan;
use colored::Colorize;
use kit as u;

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
        "logger",
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

async fn create_flow(plan: &Plan) {
    let Plan {
        env,
        functions,
        name,
        roles,
        logs,
        routes,
        events,
        flow,
        mutations,
        queues,
        ..
    } = plan;

    role::create(&env, roles.clone()).await;
    function::create(&env, functions.clone()).await;
    if should_update_layers() {
        function::update_layers(&env, functions.clone()).await;
    }
    match flow {
        Some(f) => {
            flow::create(&env, f.clone()).await;
            let sfn_name = name;
            let sfn_arn = &env.sfn_arn(&sfn_name);
            flow::enable_logs(&env, sfn_arn, logs.clone()).await;
            route::create(&env, sfn_arn, &f.default_role, routes.clone()).await;
        }
        None => {
            let role_name = "tc-base-api-role";
            let role_arn = &env.role_arn(&role_name);
            route::create(&env, "", role_arn, routes.clone()).await;
        }
    }

    match mutations {
        Some(m) => mutation::create(&env, m.clone()).await,
        None => (),
    }

    queue::create(&env, queues.to_vec()).await;
    event::create(&env, events.clone()).await;
}

async fn create_function(plan: &Plan) {
    let Plan {
        env,
        functions,
        roles,
        ..
    } = plan;
    role::create(&env, roles.clone()).await;
    function::create(&env, functions.clone()).await;
}

pub async fn create(plan: Plan) {
    let Plan {
        env,
        kind,
        namespace,
        version,
        sandbox,
        ..
    } = plan.clone();

    println!(
        "Creating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.name.cyan(),
        &env.name.blue(),
        &version
    );

    if &kind == "step-function" || &kind == "state-machine" {
        create_flow(&plan).await;
    } else {
        create_function(&plan).await;
    }
}

pub async fn update(plan: Plan) {
    let Plan {
        env,
        namespace,
        version,
        functions,
        flow,
        mutations,
        sandbox,
        events,
        queues,
        ..
    } = plan;

    println!(
        "Updating functor {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.name.cyan(),
        &env.name.blue(),
        &version
    );

    function::update_code(&env, functions.clone()).await;
    match flow {
        Some(f) => flow::create(&env, f).await,
        None => (),
    }
    match mutations {
        Some(m) => mutation::create(&env, m.clone()).await,
        None => (),
    }
    event::create(&env, events.clone()).await;
    queue::create(&env, queues).await;
}

pub async fn update_component(plan: Plan, component: Option<String>) {
    let component = maybe_component(component);
    let Plan {
        env,
        version,
        namespace,
        sandbox,
        functions,
        events,
        routes,
        flow,
        name,
        roles,
        mutations,
        schedules,
        queues,
        ..
    } = plan;

    let sfn_name = name;
    let sfn_arn = env.sfn_arn(&sfn_name);

    println!(
        "Updating functor {}@{}.{}/{}/{}",
        &namespace.green(),
        &sandbox.name.cyan(),
        &env.name.blue(),
        &version,
        &component
    );

    match component.as_str() {
        "events" => {
            event::create(&env, events).await;
        }

        "functions" => {
            function::create(&env, functions).await;
        }

        "layers" => {
            function::update_layers(&env, functions).await;
        }

        "roles" => {
            role::update(&env, roles).await;
        }

        "routes" => match flow {
            Some(f) => {
                route::create(&env, &sfn_arn, &f.default_role, routes).await;
            }
            None => {
                let role_name = "tc-base-api-role";
                let role_arn = &env.role_arn(&role_name);
                route::create(&env, "", role_arn, routes).await;
            }
        },

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

        "mutations" => match mutations {
            Some(m) => mutation::create(&env, m).await,
            None => (),
        },

        "schedules" => schedule::create(&env, &namespace, schedules).await,

        "queues" => queue::create(&env, queues).await,

        "all" => {
            role::create(&env, roles).await;
            function::create(&env, functions.clone()).await;
            match flow {
                Some(f) => flow::create(&env, f).await,
                None => (),
            }
            function::update_vars(&env, functions.clone()).await;
            function::update_tags(&env, functions).await;
        }

        _ => {
            if u::file_exists(&component) {
                let c = u::strip(&component, "/").replace("_", "-");
                match functions.get(&c) {
                    Some(f) => {
                        let tasks = f.clone().tasks;
                        let build = tasks.get("build").unwrap();
                        let dir = f.clone().dir.unwrap();
                        u::sh(build, &dir);

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

pub async fn delete(plan: Plan) {
    let Plan {
        namespace,
        env,
        functions,
        flow,
        sandbox,
        mutations,
        roles,
        routes,
        version,
        queues,
        ..
    } = plan;

    println!(
        "Deleting functor: {}@{}.{}/{}",
        &namespace.green(),
        &sandbox.name.cyan(),
        &env.name.blue(),
        &version
    );

    match flow {
        Some(f) => {
            let sfn_name = f.clone().name;
            let sfn_arn = env.sfn_arn(&sfn_name);
            flow::disable_logs(&env, &sfn_arn).await;
            flow::delete(&env, f).await;
        }
        None => println!("No flow defined, skipping"),
    }
    function::delete(&env, functions).await;
    role::delete(&env, roles).await;
    route::delete(&env, "", "", routes).await;
    match mutations {
        Some(m) => mutation::delete(&env, m.clone()).await,
        None => (),
    }
    queue::delete(&env, queues).await;
}

pub async fn delete_component(plan: Plan, component: Option<String>) {
    let component = maybe_component(component);
    let Plan {
        namespace,
        env,
        functions,
        events,
        routes,
        mutations,
        schedules,
        flow,
        roles,
        sandbox,
        version,
        ..
    } = plan;

    println!(
        "Deleting functor: {}@{}.{}/{}/{}",
        &namespace.green(),
        &sandbox.name.cyan(),
        &env.name.blue(),
        &version,
        &component
    );

    match component.as_str() {
        "events" => event::delete(&env, events).await,
        "schedules" => schedule::delete(&env, &namespace, schedules).await,
        "routes" => route::delete(&env, "", "", routes).await,
        "functions" => function::delete(&env, functions).await,
        "mutations" => match mutations {
            Some(m) => mutation::delete(&env, m).await,
            None => (),
        },
        "flow" => match flow {
            Some(f) => flow::delete(&env, f).await,
            None => (),
        },
        "roles" => role::delete(&env, roles).await,
        _ => {
            if u::file_exists(&component) {
                let c = u::strip(&component, "/");
                match functions.get(&c) {
                    Some(f) => {
                        let tasks = f.clone().tasks;
                        let build = tasks.get("build").unwrap();
                        let dir = f.clone().dir.unwrap();
                        u::sh(build, &dir);
                        function::delete_function(&env, f.clone()).await;
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
