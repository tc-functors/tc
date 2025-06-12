use crate::{aws::ecs, aws::ecs::TaskDef, aws::lambda};
use authorizer::Auth;
use compiler::{Function, Lang, function::Runtime, spec::function::Provider};
use std::collections::HashMap;

async fn create_container(auth: &Auth, function: &Function) -> String {
    let Runtime {
        cluster,
        role,
        uri,
        memory_size,
        cpu,
        handler,
        network,
        ..
    } = &function.runtime;
    let fn_name = &function.name;

    let subnets = match network {
        Some(s) => s.subnets.clone(),
        _ => vec![],
    };

    let mem = memory_size.unwrap().to_string();
    let cpu = cpu.unwrap().to_string();

    let client = ecs::make_client(auth).await;

    let tdf = TaskDef::new(&fn_name, &role.arn, &mem, &cpu);
    let cdf = ecs::make_cdf(&fn_name, &uri, &handler);
    let net = ecs::make_network_config(subnets);
    println!("Creating task def {}", fn_name);
    let taskdef_arn = ecs::create_taskdef(&client, tdf, cdf).await;

    let cluster = ecs::find_or_create_cluster(&client, &cluster).await;

    // create service or run-task

    println!("Run ecs task {}", &fn_name);
    ecs::run_task(&client, &cluster, &fn_name, &taskdef_arn, net).await;
    taskdef_arn
}

pub async fn make_lambda(auth: &Auth, f: Function) -> lambda::Function {
    let client = lambda::make_client(auth).await;
    let package_type = &f.runtime.package_type;

    let uri = f.runtime.uri;

    let (size, blob, code) = lambda::make_code(package_type, &uri);
    let vpc_config = match f.runtime.network {
        Some(s) => Some(lambda::make_vpc_config(s.subnets, s.security_groups)),
        _ => None,
    };
    let filesystem_config = match f.runtime.fs {
        Some(s) => Some(vec![lambda::make_fs_config(&s.arn, &s.mount_point)]),
        _ => None,
    };

    let arch = lambda::make_arch(&f.runtime.lang.to_str());
    let runtime = match package_type.as_ref() {
        "zip" => Some(lambda::make_runtime(&f.runtime.lang.to_str())),
        _ => None,
    };

    let handler = match package_type.as_ref() {
        "zip" => Some(f.runtime.handler),
        _ => None,
    };

    let layers = match package_type.as_ref() {
        "zip" => Some(f.runtime.layers),
        _ => None,
    };

    let snap_start = lambda::make_snapstart(f.runtime.snapstart);

    lambda::Function {
        client: client,
        name: f.fqn,
        actual_name: f.actual_name,
        description: f.description,
        code: code,
        code_size: size,
        blob: blob,
        role: f.runtime.role.arn,
        runtime: runtime,
        handler: handler,
        timeout: f.runtime.timeout.expect("Timeout error"),
        uri: uri,
        snap_start: snap_start,
        memory_size: f.runtime.memory_size.expect("memory error"),
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

pub async fn create_lambda(auth: &Auth, f: &Function) -> String {
    match f.runtime.package_type.as_ref() {
        "zip" => {
            let lambda = make_lambda(&auth, f.clone()).await;
            let id = lambda.clone().create_or_update().await;
            if f.runtime.snapstart {
                lambda.publish_version().await;
            }
            id
        }
        _ => {
            let lambda = make_lambda(&auth, f.clone()).await;
            lambda.clone().create_or_update().await
        }
    }
}

pub async fn create_function(profile: String, role_arn: Option<String>, f: Function) -> String {
    let auth = Auth::new(Some(profile), role_arn).await;

    match f.runtime.provider {
        Provider::Lambda => create_lambda(&auth, &f).await,
        Provider::Fargate => create_container(&auth, &f).await,
    }
}

pub async fn create(auth: &Auth, fns: HashMap<String, Function>) {
    match std::env::var("TC_SYNC_CREATE") {
        Ok(_) => {
            for (_, function) in fns {
                let p = auth.name.to_string();
                let role = auth.assume_role.to_owned();
                create_function(p, role, function).await;
            }
        }

        Err(_) => {
            let mut tasks = vec![];
            for (_, function) in fns {
                let p = auth.name.to_string();
                let role = auth.assume_role.to_owned();
                let h = tokio::spawn(async move {
                    create_function(p, role, function).await;
                });
                tasks.push(h);
            }
            for task in tasks {
                let _ = task.await;
            }
        }
    }
}

pub async fn update_code(auth: &Auth, fns: HashMap<String, Function>) {
    let mut tasks = vec![];
    for (_, function) in fns {
        let p = auth.name.to_string();
        let role = auth.assume_role.to_owned();
        let h = tokio::spawn(async move {
            create_function(p, role, function).await;
        });
        tasks.push(h);
    }
    for task in tasks {
        let _ = task.await;
    }
}

pub async fn delete_function(auth: &Auth, f: Function) {
    let function = make_lambda(auth, f).await;
    function.clone().delete().await.unwrap();
}

pub async fn delete(auth: &Auth, fns: HashMap<String, Function>) {
    for (_name, function) in fns {
        match function.runtime.package_type.as_ref() {
            "zip" => {
                let function = make_lambda(auth, function).await;
                function.clone().delete().await.unwrap();
            }
            _ => {
                let function = make_lambda(auth, function).await;
                function.clone().delete().await.unwrap();
            }
        }
    }
}

pub async fn update_layers(auth: &Auth, fns: HashMap<String, Function>) {
    for (_, f) in fns {
        let function = make_lambda(auth, f.clone()).await;
        let arn = auth.lambda_arn(&f.fqn);
        let _ = function.update_layers(&arn).await;
    }
}

pub async fn update_vars(auth: &Auth, funcs: HashMap<String, Function>) {
    for (_, f) in funcs {
        let memory_size = f.runtime.memory_size.expect("memory error");
        println!("mem {}", memory_size);
        let function = make_lambda(auth, f.clone()).await;
        let _ = function.update_vars().await;
    }
}

pub async fn update_concurrency(auth: &Auth, funcs: HashMap<String, Function>) {
    for (_, f) in funcs {
        let function = make_lambda(auth, f.clone()).await;

        match f.runtime.provisioned_concurrency {
            Some(n) => function.clone().update_provisioned_concurrency(n).await,
            None => (),
        };

        match f.runtime.reserved_concurrency {
            Some(n) => function.update_reserved_concurrency(n).await,
            None => (),
        };
    }
}

pub async fn update_tags(auth: &Auth, funcs: HashMap<String, Function>) {
    let client = lambda::make_client(auth).await;
    for (_, f) in funcs {
        let arn = auth.lambda_arn(&f.fqn);
        lambda::update_tags(client.clone(), &arn, f.runtime.tags.clone()).await;
    }
}

pub async fn update_runtime_version(auth: &Auth, fns: HashMap<String, Function>) {
    let client = lambda::make_client(auth).await;
    match std::env::var("TC_LAMBDA_RUNTIME_VERSION") {
        Ok(v) => {
            for (_, f) in fns {
                if f.runtime.lang.to_lang() == Lang::Ruby {
                    let _ = lambda::update_runtime_management_config(&client, &f.name, &v).await;
                }
            }
        }
        Err(_) => println!("TC_LAMBDA_RUNTIME_VERSION env var is not set"),
    }
}
