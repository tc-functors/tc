use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::{Context, Topology};
use crate::resolver::{FunctionSpec, RuntimeSpec, Vars};
use aws::Env;
use kit as u;
use kit::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Network {
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerTask {
    pub name: String,
    pub cluster: String,
    pub task_role_arn: String,
    pub image_uri: String,
    pub command: String,
    pub network_mode: String,
    pub subnets: Vec<String>,
    pub cpu: String,
    pub mem: String,
}

async fn make_container_task(
    context: &Context,
    fqn: &str,
    role: &str,
    image_uri: &str,
    command: &str
) -> ContainerTask {

    let Context { config, env, .. } = context;
    let subnets = &config.ecs.subnets;
    let cluster = &config.ecs.cluster;

    ContainerTask {
        name: s!(fqn),
        cluster: cluster.to_string(),
        task_role_arn: env.role_arn(role),
        network_mode: s!("Awsvpc"),
        image_uri: s!(image_uri),
        command: command.to_owned(),
        subnets: subnets.clone(),
        cpu: s!("1024"),
        mem: s!("2048")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Runtime {
    pub lang: String,
    pub handler: String,
    pub package_type: String,
    pub image: String,
    pub layers: Vec<String>,
    pub timeout: i32,
    pub memory_size: i32,
    pub network: Option<Network>,
    pub provisioned_concurrency: Option<i32>,
    pub environment: HashMap<String, String>,
    pub tags: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSystem {
    pub arn: String,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
    pub name: String,
    pub actual_name: String,
    pub version: String,
    pub revision: String,
    pub description: Option<String>,
    pub role: String,
    pub runtime: Runtime,
    pub container_task: Option<ContainerTask>,
    pub fs: Option<FileSystem>,
    pub uri: Option<String>,
    pub dir: Option<String>,
    pub tasks: HashMap<String, String>,
}

async fn make_network(
    context: &Context,
    assets: &HashMap<String, String>,
    network: &HashMap<String, Vec<String>>,
) -> Option<Network> {
    let Context { env, config, .. } = context;

    if !network.is_empty() {
        let given_subnets = network.get("subnets").unwrap();
        let given_sgs = network.get("security_groups").unwrap();
        let mut subnet_xs: Vec<String> = vec![];
        let mut sgs_xs: Vec<String> = vec![];
        for sn in given_subnets {
            if !&sn.starts_with("subnet") {
                let subnets = env.subnets(&sn).await;
                for s in subnets {
                    subnet_xs.push(s);
                }
            } else {
                subnet_xs.push(sn.to_string());
            }
        }
        for sg in given_sgs {
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
    } else if !assets.get("DEPS_PATH").unwrap().is_empty() {
        let given_subnet = &config.efs.subnets;
        let given_sg = &config.efs.security_group;
        let subnets = env.subnets(given_subnet).await;
        let security_groups = env.security_groups(&given_sg).await;
        let net = Network {
            subnets: subnets,
            security_groups: security_groups,
        };
        Some(net)
    } else {
        None
    }
}

fn find_ap_name(context: &Context) -> String {
    let Context {
        sandbox, config, ..
    } = context;
    match std::env::var("TC_EFS_AP") {
        Ok(t) => t,
        Err(_) => {
            if sandbox == "stable" {
                s!(&config.efs.stable_ap)
            } else {
                s!(&config.efs.dev_ap)
            }
        }
    }
}

async fn make_fs(env: &Env, context: &Context) -> Option<FileSystem> {
    let Context { config, .. } = context;
    let ap_name = find_ap_name(context);

    println!("Updating fs: {}", &ap_name);
    let arn = env.access_point_arn(&ap_name).await;
    match arn {
        Some(a) => {
            let fs = FileSystem {
                arn: a,
                mount_point: config.lambda.fs_mountpoint.to_owned(),
            };
            Some(fs)
        }
        _ => None,
    }
}

fn asset_vars(
    deps_path: Option<String>,
    base_deps_path: Option<String>,
    asset_path: Option<String>,
) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    let base_deps_path = u::maybe_string(base_deps_path, "/var/python");
    let model_path = u::maybe_string(asset_path, "/var/python");
    match deps_path {
        Some(path) => {
            h.insert(
                s!("PYTHONPATH"),
                format!(
                    "/opt/python:/var/runtime:{}/python:{}/python:{}",
                    &base_deps_path, &path, &model_path
                ),
            );
            h.insert(s!("LD_LIBRARY_PATH"), format!("/var/lang/lib:/lib64:/usr/lib64:/var/runtime:/var/runtime/lib:/var/task:/var/task/lib:/opt/lib:{}/lib", &path));
        }
        _ => (),
    }
    h
}

fn find_deps_path(default: Option<String>) -> Option<String> {
    match std::env::var("TC_EFS_DEPS") {
        Ok(d) => Some(d),
        Err(_) => default,
    }
}

async fn make_env_vars(
    context: &Context,
    vars: Vars,
    assets: &HashMap<String, String>,
) -> HashMap<String, String> {
    let Context { env, resolve, .. } = context;
    let mut env_vars = vars.clone().environment;
    if *resolve {
        env_vars = env.resolve_vars(vars.clone().environment).await;
    }
    env_vars.extend(assets.to_owned());
    let default_path = assets.get("DEPS_PATH");
    let deps_path = find_deps_path(default_path.cloned());
    let avars = asset_vars(
        deps_path,
        assets.get("BASE_DEPS_PATH").cloned(),
        assets.get("MODEL_PATH").cloned(),
    );
    env_vars.extend(avars);
    env_vars
}

fn make_assets(context: &Context, assets: &HashMap<String, Value>) -> HashMap<String, String> {
    let mut ats: HashMap<String, String> = HashMap::new();
    let deps_path = find_deps_path(Some(u::value_to_string(assets.get("DEPS_PATH"))));

    let base_deps_path = u::value_to_string(assets.get("BASE_DEPS_PATH"));
    if let Some(path) = deps_path {
        ats.insert(s!("DEPS_PATH"), context.render(&path));
    }

    ats.insert(s!("BASE_DEPS_PATH"), context.render(&base_deps_path));
    ats.insert(
        s!("MODEL_PATH"),
        u::value_to_string(assets.get("MODEL_PATH")),
    );
    ats
}

async fn resolve_layers(context: &Context, layers: Vec<String>) -> Vec<String> {
    let Context {
        env,
        resolve,
        sandbox,
        ..
    } = context;
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
            if *resolve {
                xs.push(env.resolve_layer(&layer).await)
            }
        }
    }
    xs
}

async fn make_runtime(
    context: &Context,
    fqn: &str,
    funspec: &FunctionSpec,
    tags: &HashMap<String, String>,
) -> Runtime {
    let FunctionSpec {
        runtime,
        dir,
        vars_file,
        assets,
        ..
    } = funspec;
    let RuntimeSpec { lang, .. } = runtime;
    let vars = Vars::new(context, vars_file.to_owned(), lang, fqn, &dir);
    let Vars {
        timeout,
        memory_size,
        provisioned_concurrency,
        network,
        ..
    } = vars.clone();

    let layers = resolve_layers(context, runtime.clone().layers).await;
    let assets = make_assets(context, assets);
    let env_vars = make_env_vars(context, vars, &assets).await;
    let network = make_network(context, &assets, &network).await;

    Runtime {
        lang: runtime.lang.to_owned(),
        handler: runtime.handler.to_owned(),
        package_type: runtime.package_type.to_owned(),
        layers: layers.to_vec(),
        image: u::empty(),
        tags: tags.clone(),
        environment: env_vars,
        timeout: timeout,
        network: network,
        provisioned_concurrency: provisioned_concurrency,
        memory_size: memory_size,
    }
}

impl Function {
    pub async fn new(
        dir: &str,
        context: &Context,
        funspec: &FunctionSpec,
        tags: &HashMap<String, String>,
    ) -> Function {
        let FunctionSpec {
            name,
            assets,
            fqn,
            version,
            revision,
            role,
            tasks,
            ..
        } = funspec;

        let fqn = context.render(&fqn);
        let Context { env, .. } = context;
        let runtime = make_runtime(context, &fqn, funspec, tags).await;
        let uri = format!("{}/lambda.zip", &dir);
        let role_name = context.render(&role.name);
        let role_arn = env.role_arn(&role_name);

        let mut fs = None;

        if !assets.is_empty() {
            fs = make_fs(env, context).await;
        }

       let container_task = make_container_task(
           context,
           &fqn,
           &funspec.container_task.task_role,
           &funspec.container_task.image_uri,
           &funspec.container_task.command
       ).await;


        Function {
            version: version.to_string(),
            revision: revision.to_string(),
            name: fqn,
            actual_name: s!(name),
            description: funspec.clone().description,
            runtime: runtime,
            role: s!(&role_arn),
            container_task: Some(container_task),
            fs: fs,
            uri: Some(uri),
            dir: Some(dir.to_string()),
            tasks: tasks.to_owned(),
        }
    }
}

pub async fn make(
    context: &Context,
    topology: &Topology,
    tags: &HashMap<String, String>,
) -> HashMap<String, Function> {
    let funspecs = &topology.functions;
    let mut functions: HashMap<String, Function> = HashMap::new();

    for (dir, funspec) in funspecs {
        let actual_name = funspec.clone().name;
        let function = Function::new(&dir, context, funspec, tags).await;
        functions.insert(actual_name, function);
    }
    functions
}
