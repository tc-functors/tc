use compiler::{Lang, BuildKind};
use composer::Function;
use kit as u;
use kit::s;
use provider::{
    Auth,
    aws::{
        lambda,
        lambda::LambdaClient as Client,
        lambda::Store,
        s3
    },
};
use std::collections::HashMap;

fn make(f: &Function, tags: &HashMap<String, String>) -> lambda::Function {
    let package_type = &f.runtime.package_type;

    let uri = &f.runtime.uri;

    let store = match std::env::var("TC_USE_ASSET_STORE") {
        Ok(_) => {
            match &f.build.kind {
                BuildKind::Inline => {
                    let (bucket, key) = s3::parts_of(&f.runtime.uri);
                    Some(
                        Store {
                            bucket: bucket,
                            key: key,
                            size: 0.to_string()
                        }
                    )
                },
                _ => None
            }
        },
        Err(_) => None
    };

    let (size, blob, code) = lambda::make_code(package_type, &uri, store.clone());
    let vpc_config = match &f.runtime.network {
        Some(s) => Some(lambda::make_vpc_config(
            s.subnets.clone(),
            s.security_groups.clone(),
        )),
        _ => None,
    };
    let filesystem_config = match &f.runtime.fs {
        Some(s) => Some(vec![lambda::make_fs_config(&s.arn, &s.mount_point)]),
        _ => None,
    };

    let arch = lambda::make_arch(&f.runtime.arch.to_str());
    let runtime = match package_type.as_ref() {
        "zip" => Some(lambda::make_runtime(&f.runtime.lang.to_str())),
        _ => None,
    };

    let handler = match package_type.as_ref() {
        "zip" => Some(f.runtime.handler.clone()),
        _ => None,
    };

    let layers = match package_type.as_ref() {
        "zip" => Some(f.runtime.layers.clone()),
        _ => None,
    };

    let snap_start = lambda::make_snapstart(f.runtime.snapstart);

    let f = f.clone();
    lambda::Function {
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
        uri: uri.to_string(),
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
        store: store
    }
}

pub async fn create(client: &Client, f: &Function, tags: &HashMap<String, String>) -> String {
    let lambda = make(f, tags);
    let maybe_current = lambda::find_config(client, &f.fqn).await;
    let id = if let Some(current) = maybe_current {
        let package_type = f.runtime.package_type.to_lowercase();
        let current_package_type = current.package_type.to_lowercase();
        if current_package_type != package_type {
            tracing::debug!(
                "Recreating function: {} -> {}",
                current_package_type,
                package_type
            );
            lambda.delete(client).await.unwrap();
            lambda.create_or_update(client).await
        } else {
            lambda.create_or_update(client).await
        }
    } else {
        lambda.clone().create_or_update(client).await
    };

    if f.runtime.snapstart {
        lambda.publish_version(client).await;
    }

    id
}

pub async fn update_tags(client: &Client, arn: &str, tags: &HashMap<String, String>) {
    lambda::update_tags(client, &arn, tags).await;
}

pub async fn update_runtime_version(client: &Client, fns: &HashMap<String, Function>) {
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

pub async fn update_layers(client: &Client, f: &Function, arn: &str) {
    let function = make(f, &HashMap::new());
    let _ = function.update_layers(client, arn).await;
}

pub async fn update_vars(client: &Client, f: &Function) {
    let memory_size = f.runtime.memory_size.expect("memory error");
    println!("mem {}", memory_size);
    let function = make(f, &HashMap::new());
    if f.runtime.package_type == "zip" || f.runtime.package_type == "Zip" {
        let _ = function.update_vars(client).await;
    } else {
        let _ = function.update_image_vars(client).await;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub code_size: String,
    pub timeout: i32,
    pub mem: i32,
    pub role: String,
    pub package_type: String,
    pub updated: String,
    pub version: String,
    pub uri: String,
}

pub async fn get_config(client: &Client, arn: &str) -> Option<Config> {
    let tags = lambda::list_tags(&client, &arn).await.unwrap();
    let config = lambda::find_config(&client, &arn).await;
    let maybe_uri = lambda::find_uri(&client, &arn).await;
    let uri = u::maybe_string(maybe_uri, "");
    if let Some(cfg) = config {
        let c = Config {
            code_size: u::file_size_human(cfg.code_size as f64),
            timeout: cfg.timeout,
            mem: cfg.mem_size,
            package_type: cfg.package_type,
            role: u::split_last(&cfg.role, "/"),
            updated: u::safe_unwrap(tags.get("updated_at")),
            version: u::safe_unwrap(tags.get("version")),
            uri: u::split_last(&uri, "/"),
        };
        Some(c)
    } else {
        None
    }
}

pub async fn freeze(client: &Client, fqn: &str, arn: &str) {
    let version = lambda::get_tag(&client, &arn, s!("version")).await;
    if &version != "0.0.1" && !&version.is_empty() {
        println!("Freezing function {} ({})", fqn, version);
        let kv = u::kv("freeze", "true");
        let _ = lambda::update_tags(&client, &arn, &kv).await;
    }
}

pub async fn unfreeze(client: &Client, fqn: &str, arn: &str) {
    let version = lambda::get_tag(&client, &arn, s!("version")).await;
    if &version != "0.0.1" && !&version.is_empty() {
        println!("Unfreezing function {} ({})", fqn, version);
        let kv = u::kv("freeze", "false");
        let _ = lambda::update_tags(&client, &arn, &kv).await;
    }
}

pub async fn is_frozen(client: &Client, arn: &str) -> bool {
    let v = lambda::get_tag(client, &arn, s!("freeze")).await;
    v == "true"
}

pub async fn delete(client: &Client, f: &Function) {
    let function = make(f, &HashMap::new());
    function.delete(client).await.unwrap();
}

pub async fn sync_role(client: &Client, function: &Function) {
    let role_arn = function.runtime.role.arn.clone();
    match lambda::update_role(client, &function.fqn, &role_arn).await {
        Ok(true) => println!(
            "Re-attaching role {} to {}",
            u::split_last(&role_arn, "/"),
            &function.name
        ),
        Ok(false) => (),
        Err(e) => println!("Failed to attach role for {}: {:?}", &function.name, e),
    }
}

pub async fn update_concurrency(client: &Client, f: &Function) {
    let function = make(f, &HashMap::new());

    if let Some(n) = f.runtime.provisioned_concurrency {
        function.update_provisioned_concurrency(client, n).await
    }
    if let Some(n) = f.runtime.reserved_concurrency {
        function.update_reserved_concurrency(client, n).await
    }
}

pub async fn update_destination(client: &Client, fqn: &str, arn: &str) {
    lambda::update_destination(client, fqn, arn).await;
}

pub async fn make_client(auth: &Auth) -> Client {
    lambda::make_client(auth).await
}
