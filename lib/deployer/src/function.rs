use compiler::{Function, Lang};
use aws::lambda;
use aws::Env;
//use aws::ecs;
//use aws::ecs::TaskDef;
use configurator::Config;
use std::collections::HashMap;

pub async fn make_lambda(env: &Env, f: Function) -> lambda::Function {
    let client = lambda::make_client(env).await;
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

pub async fn create_function(profile: String, role_arn: Option<String>, config: Config, f: Function) -> String {
    let env = Env::new(&profile, role_arn, config);
    match f.runtime.package_type.as_ref() {
        "zip" => {
            let lambda = make_lambda(&env, f.clone()).await;
            lambda.clone().create_or_update().await
        },
        _ => {
            let lambda = make_lambda(&env, f.clone()).await;
            lambda.clone().create_or_update().await
        }
    }
}

pub async fn create(env: &Env, fns: HashMap<String, Function>) {
    match std::env::var("TC_SYNC_CREATE") {
        Ok(_) => {
            for (_, function) in fns {
                let p = env.name.to_string();
                let role = env.assume_role.to_owned();
                let config = env.config.to_owned();
                create_function(p, role, config, function).await;
            }
        },

        Err(_) => {
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
        let arn = env.lambda_arn(&f.fqn);
        let _ = function.update_layers(&arn).await;
    }
}

pub async fn update_vars(env: &Env, funcs: HashMap<String, Function>) {
    for (_, f) in funcs {
        let memory_size = f.runtime.memory_size.expect("memory error");
        println!("mem {}", memory_size);
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
        let arn = env.lambda_arn(&f.fqn);
        lambda::update_tags(client.clone(), &arn, f.runtime.tags.clone()).await;
    }
}

pub async fn update_runtime_version(env: &Env, fns: HashMap<String, Function>) {
    let client = lambda::make_client(env).await;
    match std::env::var("TC_LAMBDA_RUNTIME_VERSION") {
        Ok(v) => {
            for (_, f) in fns {
                if f.runtime.lang.to_lang() == Lang::Ruby {
                    let _ = lambda::update_runtime_management_config(&client, &f.name, &v).await;
                }
            }
        }
        Err(_) => println!("TC_LAMBDA_RUNTIME_VERSION env var is not set")
    }
}
