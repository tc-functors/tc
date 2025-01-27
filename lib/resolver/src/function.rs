use std::collections::HashMap;

use super::Context;
use compiler::{Function, Runtime, Topology};
use compiler::function::runtime::{Network, FileSystem};
use kit::*;

async fn resolve_environment(ctx: &Context,
                             env_vars: HashMap<String, String>) -> HashMap<String, String> {

    let Context { env, .. } = ctx;
    // lookup by sandbox
    env.resolve_vars(env_vars).await
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

async fn resolve_runtime(ctx: &Context, runtime: &Runtime) -> Runtime {
    let Runtime { environment, layers, network, fs, .. } = runtime;
    let mut r: Runtime = runtime.clone();

    r.environment = resolve_environment(ctx, environment.clone()).await;
    r.layers = resolve_layers(ctx, layers.clone()).await;
    r.network = resolve_network(ctx, network.clone()).await;
    r.fs = resolve_fs(ctx, fs.clone()).await;
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
