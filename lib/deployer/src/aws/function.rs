use crate::role;
use compiler::spec::function::{
    Lang,
    Provider,
};
use composer::Function;
use kit as u;
use provider::{
    Auth,
    aws::lambda,
};
use std::collections::HashMap;
use tabled::Tabled;

async fn maybe_build(auth: &Auth, function: &Function) {
    let builds = builder::build(auth, function, None, None, true).await;
    builder::publish(auth, builds).await;
}

pub async fn make_lambda(
    auth: &Auth,
    f: Function,
    tags: &HashMap<String, String>,
) -> lambda::Function {
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
        tags: tags.clone(),
        layers: layers,
        vpc_config: vpc_config,
        filesystem_config: filesystem_config,
        _logging_config: None,
    }
}

pub async fn create_lambda(auth: &Auth, f: &Function, tags: &HashMap<String, String>) -> String {
    let lambda = make_lambda(&auth, f.clone(), tags).await;
    let maybe_current = lambda::find_config(&lambda.client, &f.fqn).await;
    let id = if let Some(current) = maybe_current {
        let package_type = f.runtime.package_type.to_lowercase();
        let current_package_type = current.package_type.to_lowercase();
        if current_package_type != package_type {
            tracing::debug!(
                "Recreating function: {} -> {}",
                current_package_type,
                package_type
            );
            lambda.clone().delete().await.unwrap();
            lambda.clone().create_or_update().await
        } else {
            lambda.clone().create_or_update().await
        }
    } else {
        lambda.clone().create_or_update().await
    };

    if f.runtime.snapstart {
        lambda.publish_version().await;
    }
    if let Some(target) = &f.target {
        println!("Creating target {}", &target.entity.to_str());
        lambda::update_destination(&lambda.client, &f.fqn, &target.arn).await;
    }

    id
}

async fn create_function(auth: &Auth, f: Function, tags: &HashMap<String, String>) -> String {
    maybe_build(auth, &f).await;
    match f.runtime.provider {
        Provider::Lambda => create_lambda(&auth, &f, tags).await,
        Provider::Fargate => todo!(),
    }
}

fn should_sync(sync: bool) -> bool {
    sync || match std::env::var("TC_SYNC_CREATE") {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn create(
    auth: &Auth,
    fns: &HashMap<String, Function>,
    tags: &HashMap<String, String>,
    sync: bool,
) {
    if should_sync(sync) {
        for (_, function) in fns.clone() {
            let a = auth.clone();
            let f = function.clone();
            create_function(&a, f, tags).await;
        }
    } else {
        let mut tasks = vec![];
        for (_, function) in fns.clone() {
            let a = auth.clone();
            let f = function.clone();
            let t = tags.clone();
            let h = tokio::spawn(async move {
                create_function(&a, f, &t).await;
            });
            tasks.push(h);
        }
        for task in tasks {
            let _ = task.await;
        }
    }
}

pub async fn update_code(
    auth: &Auth,
    fns: &HashMap<String, Function>,
    tags: &HashMap<String, String>,
) {
    let mut tasks = vec![];
    for (_, function) in fns.clone() {
        let a = auth.clone();
        let f = function.clone();
        let t = tags.clone();
        let h = tokio::spawn(async move {
            create_function(&a, f, &t).await;
        });
        tasks.push(h);
    }
    for task in tasks {
        let _ = task.await;
    }
}

pub async fn delete(auth: &Auth, fns: &HashMap<String, Function>) {
    for (_name, function) in fns {
        match function.runtime.package_type.as_ref() {
            "zip" => {
                let function = make_lambda(auth, function.clone(), &HashMap::new()).await;
                function.clone().delete().await.unwrap();
            }
            _ => {
                let function = make_lambda(auth, function.clone(), &HashMap::new()).await;
                function.clone().delete().await.unwrap();
            }
        }
    }
}

// component updates
async fn update_layers(auth: &Auth, fns: &HashMap<String, Function>) {
    for (_, f) in fns {
        let function = make_lambda(auth, f.clone(), &HashMap::new()).await;
        let arn = auth.lambda_arn(&f.fqn);
        let _ = function.update_layers(&arn).await;
    }
}

async fn update_vars(auth: &Auth, funcs: &HashMap<String, Function>) {
    for (_, f) in funcs {
        let memory_size = f.runtime.memory_size.expect("memory error");
        println!("mem {}", memory_size);
        let function = make_lambda(auth, f.clone(), &HashMap::new()).await;
        if f.runtime.package_type == "zip" || f.runtime.package_type == "Zip" {
            let _ = function.update_vars().await;
        } else {
            let _ = function.update_image_vars().await;
        }
    }
}
async fn update_roles(auth: &Auth, funcs: &HashMap<String, Function>) {
    let mut roles: HashMap<String, composer::Role> = HashMap::new();
    for (_, f) in funcs {
        let kind = f.runtime.role.kind.clone();
        let role_kind: &str = &kind.to_str();
        if role_kind != "base" || role_kind != "provided" {
            roles.insert(f.runtime.role.name.clone(), f.runtime.role.clone());
        }
    }
    role::create_or_update(auth, &roles, &HashMap::new()).await;
}

async fn update_concurrency(auth: &Auth, funcs: &HashMap<String, Function>) {
    for (_, f) in funcs {
        let function = make_lambda(auth, f.clone(), &HashMap::new()).await;

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

async fn update_tags(
    auth: &Auth,
    funcs: &HashMap<String, Function>,
    tags: &HashMap<String, String>,
) {
    let client = lambda::make_client(auth).await;
    for (_, f) in funcs {
        let arn = auth.lambda_arn(&f.fqn);
        lambda::update_tags(client.clone(), &arn, tags.clone()).await;
    }
}

async fn update_runtime_version(auth: &Auth, fns: &HashMap<String, Function>) {
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

pub async fn update_dir(
    auth: &Auth,
    functions: &HashMap<String, Function>,
    dir: &str,
    tags: &HashMap<String, String>,
) {
    if kit::file_exists(dir) {
        match functions.get(dir) {
            Some(f) => {
                let a = auth.clone();
                maybe_build(&a, &f).await;
                create_function(&a, f.clone(), tags).await;
            }
            None => panic!("No valid function found"),
        }
    }
}

pub async fn update(
    auth: &Auth,
    functions: &HashMap<String, Function>,
    tags: &HashMap<String, String>,
    component: &str,
) {
    match component {
        "layers" => update_layers(&auth, functions).await,
        "vars" => update_vars(&auth, functions).await,
        "concurrency" => update_concurrency(&auth, functions).await,
        "runtime" => update_runtime_version(&auth, functions).await,
        "tags" => update_tags(&auth, functions, tags).await,
        "roles" => update_roles(&auth, functions).await,
        _ => update_dir(&auth, functions, component, tags).await,
    }
}

// list

#[derive(Tabled, Clone, Debug, PartialEq)]
pub struct Record {
    pub name: String,
    pub code_size: String,
    pub timeout: i32,
    pub mem: i32,
    pub role: String,
    pub package_type: String,
    pub updated: String,
    pub version: String,
    pub uri: String,
}

pub async fn list(auth: &Auth, fns: &HashMap<String, Function>) -> Vec<Record> {
    let client = lambda::make_client(auth).await;
    let mut rows: Vec<Record> = vec![];
    for (_, f) in fns {
        let arn = auth.lambda_arn(&f.fqn);
        let tags = lambda::list_tags(&client, &arn).await.unwrap();

        let config = lambda::find_config(&client, &arn).await;
        let maybe_uri = lambda::find_uri(&client, &arn).await;
        let uri = u::maybe_string(maybe_uri, "");
        match config {
            Some(cfg) => {
                let row = Record {
                    name: f.name.clone(),
                    code_size: u::file_size_human(cfg.code_size as f64),
                    timeout: cfg.timeout,
                    mem: cfg.mem_size,
                    package_type: cfg.package_type,
                    role: u::split_last(&cfg.role, "/"),
                    updated: u::safe_unwrap(tags.get("updated_at")),
                    version: u::safe_unwrap(tags.get("version")),
                    uri: u::split_last(&uri, "/"),
                };
                rows.push(row);
            }
            None => (),
        }
    }
    rows
}

pub async fn create_dry_run(fns: &HashMap<String, Function>) {
    for (_, function) in fns {
        println!("Creating function: {}", &function.fqn);
    }
}
