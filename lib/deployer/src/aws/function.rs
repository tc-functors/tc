use crate::role;
use compiler::spec::function::{
    Provider,
};
use composer::{
    Function,
};
use futures::stream::FuturesUnordered;
use provider::{
    Auth,
};
use std::collections::HashMap;
pub mod lambda;
mod microvm;
mod agentcore;
use tabled::Tabled;

async fn maybe_build(auth: &Auth, function: &Function) {
    let builds = builder::build(auth, function, None, None, true).await;
    builder::publish(auth, builds).await;
}

async fn create_function(auth: &Auth, f: Function, tags: &HashMap<String, String>) -> String {
    maybe_build(auth, &f).await;
    match f.runtime.provider {
        Provider::Lambda => {
            let client = lambda::make_client(auth).await;
            lambda::create(&client, &f, tags).await
        }
        Provider::MicroVm => {
            microvm::create(auth, &f, tags).await
        },
        Provider::AgentCore => todo!(),
    }
}

fn get_chunk_size() -> usize {
    match std::env::var("TC_FUNCTION_CREATE_CONCURRENCY") {
        Ok(n) => n.parse::<usize>().unwrap(),
        Err(_) => 4,
    }
}

pub async fn create(
    auth: &Auth,
    fns: &HashMap<String, Function>,
    tags: &HashMap<String, String>,
    _sync: bool,
) {
    let names: Vec<String> = Vec::from_iter(fns.keys().cloned());
    let csize = get_chunk_size();
    let chunks: Vec<&[String]> = names.chunks(csize).collect();

    for chunk in chunks {
        let futs = FuturesUnordered::new();

        println!("Chunking functions: {:?}", &chunk.len());
        for name in chunk {
            let f = fns.get(name).unwrap().clone();
            let a = auth.clone();
            let t = tags.clone();
            let fut = tokio::spawn(async move {
                create_function(&a, f, &t).await;
            });
            futs.push(fut);
        }
        for task in futs {
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
    let client = lambda::make_client(auth).await;
    for (_name, f) in fns {
         match f.runtime.provider {
             Provider::Lambda => lambda::delete(&client, &f).await,
             Provider::MicroVm => microvm::delete(auth, &f).await,
             Provider::AgentCore => todo!()
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

pub async fn sync_roles(auth: &Auth, fns: &HashMap<String, Function>) {
    if fns.is_empty() {
        return;
    }
    let client = lambda::make_client(auth).await;

    let mut tasks = vec![];

    for (_, f) in fns.clone() {
        if !matches!(f.runtime.provider, Provider::Lambda) {
            continue;
        }

        let c = client.clone();
        let h = tokio::spawn(async move {
            lambda::sync_role(&c, &f).await
        });
        tasks.push(h);
    }
    for task in tasks {
        let _ = task.await;
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
        "layers" => update_layers(auth, functions).await,
        "vars" => update_vars(auth, functions).await,
        "concurrency" => update_concurrency(auth, functions).await,
        "runtime" => {
            let client = lambda::make_client(auth).await;
            lambda::update_runtime_version(&client, functions).await;
        }
        "tags" => update_tags(auth, functions, tags).await,
        "roles" => update_roles(auth, functions).await,
        _ => update_dir(auth, &functions, component, tags).await,
    }
}

pub async fn create_dry_run(fns: &HashMap<String, Function>) {
    for (_, function) in fns {
        println!("Creating function: {}", &function.fqn);
    }
}

pub async fn is_frozen(auth: &Auth, fqn: &str) -> bool {
    let client = lambda::make_client(auth).await;
    let arn = auth.lambda_arn(fqn);
    lambda::is_frozen(&client, &arn).await
}

pub async fn freeze(auth: &Auth, fqn: &str) {
    let client = lambda::make_client(auth).await;
    let arn = auth.lambda_arn(fqn);
    lambda::freeze(&client, fqn, &arn).await
}

pub async fn unfreeze(auth: &Auth, fqn: &str) {
    let client = lambda::make_client(auth).await;
    let arn = auth.lambda_arn(fqn);
    lambda::unfreeze(&client, fqn, &arn).await
}

pub async fn update_tags(
    auth: &Auth,
    funcs: &HashMap<String, Function>,
    tags: &HashMap<String, String>,
) {
    let client = lambda::make_client(auth).await;
    for (_, f) in funcs {
        match f.runtime.provider {
            Provider::Lambda => {
                let arn = auth.lambda_arn(&f.fqn);
                lambda::update_tags(&client, &arn, tags).await;
            },
            Provider::MicroVm => todo!(),
            Provider::AgentCore => todo!()
        }

    }
}

async fn update_layers(auth: &Auth, fns: &HashMap<String, Function>) {
    let client = lambda::make_client(auth).await;
    for (_, f) in fns {
        match f.runtime.provider {
            Provider::Lambda => {
                let arn = auth.lambda_arn(&f.fqn);
                lambda::update_layers(&client, &f, &arn).await;
            },
            _ => todo!()
        }
    }
}

async fn update_vars(auth: &Auth, fns: &HashMap<String, Function>) {
    let client = lambda::make_client(auth).await;
    for (_, f) in fns {
        match f.runtime.provider {
            Provider::Lambda => lambda::update_vars(&client, &f).await,
            _ => todo!()
        }
    }
}

async fn update_concurrency(auth: &Auth, fns: &HashMap<String, Function>) {
    let client = lambda::make_client(auth).await;
    for (_, f) in fns {
        match f.runtime.provider {
            Provider::Lambda => lambda::update_concurrency(&client, f).await,
            _ => ()
        }
    }
}

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
    for (name, f) in fns {
        match f.runtime.provider {
            Provider::Lambda => {
                let arn = auth.lambda_arn(&f.fqn);
                let maybe_cfg = lambda::get_config(&client, &arn).await;
                if let Some(cfg) = maybe_cfg {
                    let row = Record {
                        name: name.clone(),
                        code_size: cfg.code_size,
                        timeout: cfg.timeout,
                        mem: cfg.mem,
                        package_type: cfg.package_type,
                        role: cfg.role,
                        updated: cfg.updated,
                        version: cfg.version,
                        uri: cfg.uri,
                    };
                    rows.push(row);
                }
            },
            _ => ()
        }
    }
    rows
}


// transducers
