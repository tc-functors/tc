use super::Context;
use crate::aws;
use authorizer::Auth;
use composer::{
    Function,
    InfraSpec,
    Runtime,
    Topology,
    function::runtime::{
        FileSystem,
        Network,
    },
};
use kit::*;
use std::collections::HashMap;

// aws

async fn resolve_vars(
    auth: &Auth,
    environment: HashMap<String, String>,
) -> HashMap<String, String> {
    let client = aws::ssm::make_client(auth).await;

    let mut h: HashMap<String, String> = HashMap::new();
    for (k, v) in environment.iter() {
        if v.starts_with("ssm:/") {
            let key = kit::split_last(v, ":");
            let val = aws::ssm::get(client.clone(), &key).await.unwrap();
            h.insert(s!(k), val);
        } else {
            h.insert(s!(k), s!(v));
        }
    }
    h
}

async fn make_layer_auth(ctx: &Context) -> Auth {
    let Context { auth, config, .. } = ctx;
    let profile = config.aws.lambda.layers_profile.clone();
    auth.assume(profile.clone(), config.role_to_assume(profile))
        .await
}

async fn resolve_layer(ctx: &Context, layer_name: &str) -> String {
    let auth = make_layer_auth(ctx).await;
    let client = aws::layer::make_client(&auth).await;
    aws::layer::find_version(client, layer_name).await.unwrap()
}

async fn resolve_access_point_arn(ctx: &Context, name: &str) -> Option<String> {
    let auth = make_layer_auth(ctx).await;
    aws::efs::get_ap_arn(&auth, name).await.unwrap()
}

// arn
fn as_layer_arn(auth: &Auth, name: &str) -> String {
    format!(
        "arn:aws:lambda:{}:{}:layer:{}",
        auth.region, auth.account, name
    )
}

//
fn augment_vars(ctx: &Context, lang: &str) -> HashMap<String, String> {
    let mut hmap: HashMap<String, String> = HashMap::new();
    let profile = &ctx.auth.name;
    let sandbox = &ctx.sandbox;
    match lang {
        "ruby3.2" => {
            if sandbox != "stable" {
                hmap.insert(
                    String::from("HONEYBADGER_ENV"),
                    format!("{}-{}", profile, sandbox),
                );
            } else {
                hmap.insert(String::from("HONEYBADGER_ENV"), s!(profile));
            }
        }
        _ => {
            if sandbox != "stable" {
                hmap.insert(
                    String::from("HONEYBADGER_ENVIRONMENT"),
                    format!("{}-{}", profile, sandbox),
                );
            } else {
                hmap.insert(String::from("HONEYBADGER_ENVIRONMENT"), s!(profile));
            }
        }
    }
    hmap
}

async fn resolve_environment(
    ctx: &Context,
    lang: &str,
    default_vars: &HashMap<String, String>,
    sandbox_vars: Option<HashMap<String, String>>,
) -> HashMap<String, String> {
    let Context { auth, .. } = ctx;
    let mut default_vars = default_vars.clone();

    let augmented_vars = augment_vars(ctx, lang);
    default_vars.extend(augmented_vars);

    let combined = match sandbox_vars {
        Some(v) => {
            default_vars.extend(v);
            default_vars
        }
        None => default_vars,
    };

    resolve_vars(auth, combined.clone()).await
}

async fn resolve_fs(ctx: &Context, fs: Option<FileSystem>) -> Option<FileSystem> {
    let Context {
        sandbox, config, ..
    } = ctx;

    match fs {
        Some(f) => Some(f),
        None => {
            let ap_name = match std::env::var("TC_EFS_AP") {
                Ok(t) => t,
                Err(_) => match sandbox.as_ref() {
                    "stable" => s!(&config.aws.efs.stable_ap),
                    _ => s!(&config.aws.efs.dev_ap),
                },
            };
            let arn = resolve_access_point_arn(ctx, &ap_name).await;
            match arn {
                Some(a) => {
                    let fs = FileSystem {
                        arn: a,
                        mount_point: config.aws.lambda.fs_mountpoint.to_owned(),
                    };
                    Some(fs)
                }
                _ => None,
            }
        }
    }
}

async fn resolve_network(ctx: &Context, network: Option<Network>) -> Option<Network> {
    let Context { auth, config, .. } = ctx;

    match network {
        Some(net) => Some(net),
        None => {
            let cfg = &config.aws.efs.network;
            let cfg_net = cfg.get(&auth.name);
            match cfg_net {
                Some(netc) => {
                    let net = Network {
                        subnets: netc.subnets.clone(),
                        security_groups: netc.security_groups.clone(),
                    };
                    Some(net)
                }
                None => None,
            }
        }
    }
}

async fn resolve_layers(ctx: &Context, layers: Vec<String>) -> Vec<String> {
    let Context { auth, sandbox, .. } = ctx;
    let mut xs: Vec<String> = vec![];

    for layer in layers {
        if layer.contains(":") {
            xs.push(as_layer_arn(&auth, &layer))
        } else if *sandbox != "stable" {
            let name = match std::env::var("TC_USE_STABLE_LAYERS") {
                Ok(_) => layer,
                Err(_) => format!("{}-dev", &layer),
            };
            xs.push(resolve_layer(ctx, &name).await);
        } else {
            xs.push(resolve_layer(ctx, &layer).await)
        }
    }
    xs
}

fn augment_infra_spec(default: &InfraSpec, s: &InfraSpec) -> InfraSpec {
    InfraSpec {
        memory_size: match s.memory_size {
            Some(p) => {
                if p != 128 {
                    Some(p)
                } else {
                    default.memory_size
                }
            }
            None => default.memory_size,
        },
        timeout: match s.timeout {
            Some(p) => {
                if p != 300 {
                    Some(p)
                } else {
                    default.timeout
                }
            }
            None => default.timeout,
        },
        environment: match s.environment.clone() {
            Some(p) => {
                let mut def = default.environment.clone().unwrap();
                def.extend(p);
                Some(def)
            }
            None => default.environment.clone(),
        },
        image_uri: None,
        network: None,
        filesystem: None,
        provisioned_concurrency: match s.provisioned_concurrency {
            Some(p) => Some(p),
            None => default.provisioned_concurrency,
        },
        reserved_concurrency: match s.reserved_concurrency {
            Some(p) => Some(p),
            None => default.reserved_concurrency,
        },
        tags: None,
    }
}

fn get_infra_spec(
    infra_spec: &HashMap<String, InfraSpec>,
    profile: &str,
    sandbox: &str,
) -> InfraSpec {
    let profile_specific = infra_spec.get(profile);
    let sandbox_specific = infra_spec.get(sandbox);
    let default = infra_spec.get("default").unwrap();

    if let Some(s) = profile_specific {
        return augment_infra_spec(&default, s);
    }
    if let Some(s) = sandbox_specific {
        return augment_infra_spec(&default, s);
    }

    default.clone()
}

async fn resolve_runtime(ctx: &Context, runtime: &Runtime) -> Runtime {
    let Context { auth, sandbox, .. } = ctx;

    let Runtime {
        layers,
        network,
        fs,
        infra_spec,
        enable_fs,
        ..
    } = runtime;
    let mut r: Runtime = runtime.clone();

    let actual_infra = get_infra_spec(infra_spec, &auth.name, sandbox);
    let InfraSpec {
        memory_size,
        timeout,
        environment,
        ..
    } = actual_infra;

    r.memory_size = memory_size;
    r.timeout = timeout;
    r.environment = resolve_environment(
        ctx,
        &runtime.lang.to_str(),
        &runtime.environment,
        environment,
    )
    .await;
    if !layers.is_empty() {
        r.layers = resolve_layers(ctx, layers.clone()).await;
    }
    if *enable_fs {
        r.network = resolve_network(ctx, network.clone()).await;
        r.fs = resolve_fs(ctx, fs.clone()).await;
    }
    r.infra_spec = HashMap::new();
    r
}

pub async fn resolve(ctx: &Context, topology: &Topology, _dirty: bool) -> HashMap<String, Function> {
    let fns = &topology.functions;
    let mut functions: HashMap<String, Function> = HashMap::new();

    for (name, f) in fns {
        let mut fu: Function = f.clone();
        tracing::debug!("Resolving function {}", &name);
        fu.runtime = resolve_runtime(ctx, &f.runtime).await;
        functions.insert(name.to_string(), fu.clone());
    }
    functions
}
