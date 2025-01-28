use std::collections::HashMap;

use super::Context;
use compiler::{Function, Runtime, Topology, RuntimeInfraSpec};
use compiler::function::runtime::{Network, FileSystem};
use kit::*;

async fn resolve_environment(
    ctx: &Context,
    default_vars: &HashMap<String, String>,
    sandbox_vars: Option<HashMap<String, String>>
) -> HashMap<String, String> {

    let Context { env, .. } = ctx;
    let mut default_vars = default_vars.clone();

    let combined = match sandbox_vars {
        Some(v) => {
            default_vars.extend(v);
            default_vars
        }
        None => default_vars
    };

    env.resolve_vars(combined.clone()).await
}

async fn resolve_fs(ctx: &Context, fs: Option<FileSystem>) -> Option<FileSystem> {

    let Context { env, sandbox, .. } = ctx;

    match fs {
        Some(f) => Some(f),
        None => {
            let ap_name =  match sandbox.as_ref() {
                "stable" => s!(&env.config.aws.efs.stable_ap),
                _ => s!(&env.config.aws.efs.dev_ap)
            };

            let arn = env.access_point_arn(&ap_name).await;
            match arn {
                Some(a) => {
                    let fs = FileSystem {
                        arn: a,
                        mount_point: env.config.aws.lambda.fs_mountpoint.to_owned(),
                    };
                    Some(fs)
                }
                _ => None,
            }

        }
    }
}

async fn resolve_network(ctx: &Context, network: Option<Network>) -> Option<Network> {

    let Context { env, .. } = ctx;

    match network {

        Some(net) => {
            let subnet_tags = net.subnets;
            let sg_tags = net.security_groups;
            let mut subnet_xs: Vec<String> = vec![];
            let mut sgs_xs: Vec<String> = vec![];
            for sn in subnet_tags {
                if !&sn.starts_with("subnet") {
                    let subnets = env.subnets(&sn).await;
                    for s in subnets {
                        subnet_xs.push(s);
                    }
                } else {
                    subnet_xs.push(sn.to_string());
                }
            }
            for sg in sg_tags {
                if !&sg.starts_with("sg") {
                    let sgs = env.security_groups(&sg).await;
                    for s in sgs {
                        sgs_xs.push(s);
                    }
                } else {
                    sgs_xs.push(sg.to_string());
                }
            }
            let net = Network {
                subnets: subnet_xs,
                security_groups: sgs_xs,
            };
            Some(net)
        },

        None => {
            let given_subnet = &env.config.aws.efs.subnets;
            let given_sg = &env.config.aws.efs.security_group;
            let subnets = env.subnets(given_subnet).await;
            let security_groups = env.security_groups(&given_sg).await;
            let net = Network {
                subnets: subnets,
                security_groups: security_groups,
            };
            Some(net)
        }
    }
}

async fn resolve_layers(ctx: &Context, layers: Vec<String>) -> Vec<String> {

    let Context { env, sandbox, .. } = ctx;
    let mut xs: Vec<String> = vec![];

    for layer in layers {
        if layer.contains(":") {
            xs.push(env.layer_arn(&layer))

        } else if *sandbox != "stable" {
            let name = match std::env::var("TC_USE_STABLE_LAYERS") {
                Ok(_) => layer,
                Err(_) => format!("{}-dev", &layer),
            };
            xs.push(env.resolve_layer(&name).await);
        } else {
            xs.push(env.resolve_layer(&layer).await)
        }
    }
    xs
}

fn augment_infra_spec(default: &RuntimeInfraSpec, s: &RuntimeInfraSpec) -> RuntimeInfraSpec {
    RuntimeInfraSpec {
        memory_size: match s.memory_size {
            Some(p) => Some(p),
            None => default.memory_size
        },
        timeout: match s.timeout {
            Some(p) => Some(p),
            None => default.timeout
        },
        environment: match s.environment.clone() {
            Some(mut p) => {
                p.extend(default.environment.clone().unwrap());
                Some(p.clone())
            },
            None => default.environment.clone()
        },
        image_uri: None,
        network: None,
        filesystem: None,
        provisioned_concurrency: None,
        tags: None
    }
}

fn get_infra_spec(infra_spec: &HashMap<String, RuntimeInfraSpec>, profile: &str, sandbox: &str) -> RuntimeInfraSpec {

    let profile_specific = infra_spec.get(profile);
    let sandbox_specific = infra_spec.get(sandbox);
    let default = infra_spec.get("default").unwrap();

    if let Some(s) = sandbox_specific {
        return augment_infra_spec(&default, s)
    }
    if let Some(s) = profile_specific {
        return augment_infra_spec(&default, s)
    }
    default.clone()

}

async fn resolve_runtime(ctx: &Context, runtime: &Runtime) -> Runtime {
    let Context { env, sandbox, .. } = ctx;

    let Runtime { layers, network, fs, infra_spec, .. } = runtime;
    let mut r: Runtime = runtime.clone();

    let actual_infra = get_infra_spec(infra_spec, &env.name, sandbox);
    let RuntimeInfraSpec { memory_size, timeout, environment, .. } = actual_infra;

    r.memory_size = memory_size;
    r.timeout = timeout;
    r.environment = resolve_environment(ctx, &runtime.environment, environment).await;
    r.layers = resolve_layers(ctx, layers.clone()).await;
    r.network = resolve_network(ctx, network.clone()).await;
    r.fs = resolve_fs(ctx, fs.clone()).await;
    r.infra_spec = HashMap::new();
    r
}

pub async fn resolve(ctx: &Context, topology: &Topology) -> HashMap<String, Function> {

    let fns = &topology.functions;
    let mut functions: HashMap<String, Function> = HashMap::new();

    for (dir, f) in fns {
        let mut fu: Function = f.clone();
        fu.runtime = resolve_runtime(ctx, &f.runtime).await;
        functions.insert(dir.to_string(), fu.clone());
    }
    functions
}
