use resolver::{Function, ContainerTask};
use aws::lambda;
use aws::Env;
use aws::ecs;
use aws::ecs::TaskDef;
use configurator::Config;
use kit::*;
use std::collections::HashMap;

async fn delete_container_task(env: &Env, fn_name: &str, ct: ContainerTask) {
    let client = ecs::make_client(env).await;

    let ContainerTask { cluster, .. } = ct;

    println!("Deleting ecs service {}", fn_name);
    ecs::delete_service(&client, &cluster, fn_name).await;
}

async fn create_container_task(
    env: &Env,
    fn_name: &str,
    ct: ContainerTask
) -> String {
    let ContainerTask { task_role_arn,
                        cluster, image_uri, cpu, mem,
                        subnets, command,
                        .. } = ct;

    let client = ecs::make_client(env).await;
    let tdf = TaskDef::new(fn_name, &task_role_arn, &mem, &cpu);
    let cdf = ecs::make_cdf(s!(fn_name), image_uri, command);
    let net = ecs::make_network_config(subnets);
    println!("Creating task def {}", fn_name);
    let taskdef_arn  = ecs::create_taskdef(&client, tdf, cdf).await;

    println!("Creating ecs service {}", fn_name);
    ecs::create_service(
        &client,
        &cluster,
        &fn_name,
        &taskdef_arn,
        net
    ).await;
    taskdef_arn
}

pub async fn make_lambda(env: &Env, f: Function) -> lambda::Function {
    let client = lambda::make_client(env).await;
    let package_type = &f.runtime.package_type;

    let uri = f.runtime.uri;

    let (size, blob, code) = lambda::make_code(package_type, &uri);
    let vpc_config = match f.runtime.network {
        Some(s) => Some(lambda::make_vpc_config(s.subnets, s.security_groups)),
        _ => None,
    };
    let filesystem_config = match f.fs {
        Some(s) => Some(vec![lambda::make_fs_config(&s.arn, &s.mount_point)]),
        _ => None,
    };

    let arch = lambda::make_arch(&f.runtime.lang);
    let runtime = match package_type.as_ref() {
        "zip" => Some(lambda::make_runtime(&f.runtime.lang)),
        _ => None
    };

    let handler = match package_type.as_ref() {
        "zip" => Some(f.runtime.handler),
        _ => None
    };

    let layers = match package_type.as_ref() {
        "zip" => Some(f.runtime.layers),
        _ => None
    };



    lambda::Function {
        client: client,
        name: f.name,
        actual_name: f.actual_name,
        description: f.description,
        code: code,
        code_size: size,
        blob: blob,
        role: f.role,
        runtime: runtime,
        handler: handler,
        timeout: f.runtime.timeout,
        uri: uri,
        memory_size: f.runtime.memory_size,
        package_type: lambda::make_package_type(package_type),
        environment: lambda::make_environment(f.runtime.environment),
        architecture: arch,
        tags: f.runtime.tags,
        layers: layers,
        vpc_config: vpc_config,
        filesystem_config: filesystem_config,
        logging_config: None,
    }
}

pub async fn create_function(profile: String, role_arn: Option<String>, config: Config, f: Function) -> String {
    let env = Env::new(&profile, role_arn, config);
    match f.runtime.package_type.as_ref() {
        "zip" => {
            let lambda = make_lambda(&env, f.clone()).await;
            lambda.clone().create_or_update().await
        },
        "ecs-image" | "container-task" => {
            if let Some(ct) = f.container_task {
                create_container_task(&env, &f.name, ct).await
            } else {
                panic!("There is no container task defined")
            }
        },
        _ => {
            let lambda = make_lambda(&env, f.clone()).await;
            lambda.clone().create_or_update().await
        }
    }
}

pub async fn create(env: &Env, fns: HashMap<String, Function>) {
    let mut tasks = vec![];
    for (_, function) in fns {
        let p = env.name.to_string();
        let role = env.assume_role.to_owned();
        let config = env.config.to_owned();
        let h = tokio::spawn(async move {
            create_function(p, role, config, function).await;
        });
        tasks.push(h);
    }
    for task in tasks {
        let _ = task.await;
    }
}

pub async fn update_code(env: &Env, fns: HashMap<String, Function>) {
    let mut tasks = vec![];
    for (_, function) in fns {
        let p = env.name.to_string();
        let role = env.assume_role.to_owned();
        let config = env.config.to_owned();
        let h = tokio::spawn(async move {
            create_function(p, role, config, function).await;
        });
        tasks.push(h);
    }
    for task in tasks {
        let _ = task.await;
    }
}

pub async fn delete_function(env: &Env, f: Function) {
    let function = make_lambda(env, f).await;
    function.clone().delete().await.unwrap();
}


pub async fn delete(env: &Env, fns: HashMap<String, Function>) {
    for (_name, function) in fns {

        match function.runtime.package_type.as_ref() {
            "zip" => {
                let function = make_lambda(env, function).await;
                function.clone().delete().await.unwrap();
            },
            "ecs-image" | "container-task" => {
                if let Some(ct) = function.container_task {
                    delete_container_task(&env, &function.name, ct).await
                } else {
                    panic!("There is no container task defined")
                }
            },
            _ => {
                let function = make_lambda(env, function).await;
                function.clone().delete().await.unwrap();
            }
        }
    }
}

pub async fn update_layers(env: &Env, fns: HashMap<String, Function>) {
    for (_, f) in fns {
        let function = make_lambda(env, f.clone()).await;
        let arn = env.lambda_arn(&f.name);
        let _ = function.update_layers(&arn).await;
    }
}

pub async fn update_vars(env: &Env, funcs: HashMap<String, Function>) {
    for (_, f) in funcs {
        let function = make_lambda(env, f.clone()).await;
        let _ = function.clone().update_vars().await;

        match f.runtime.provisioned_concurrency {
            Some(n) => function.update_concurrency(n).await,
            None => (),
        };
    }
}

pub async fn update_tags(env: &Env, funcs: HashMap<String, Function>) {
    let client = lambda::make_client(env).await;
    for (_, f) in funcs {
        let arn = env.lambda_arn(&f.name);
        lambda::update_tags(client.clone(), &f.name, &arn, f.runtime.tags.clone()).await;
    }
}
