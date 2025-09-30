use crate::aws::{
    role::Role,
};
use compiler::{
    spec::{
        LangRuntime,
        function::{
            AssetsSpec,
            FunctionSpec,
            runtime::Provider,
            infra::InfraSpec
        },
    },
};
use kit as u;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use safe_unwrap::safe_unwrap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Network {
    pub subnets: Vec<String>,
    pub security_groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSystem {
    pub arn: String,
    pub mount_point: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Runtime {
    pub lang: LangRuntime,
    pub provider: Provider,
    pub handler: String,
    pub package_type: String,
    pub uri: String,
    pub layers: Vec<String>,
    pub environment: HashMap<String, String>,
    pub memory_size: Option<i32>,
    pub cpu: Option<i32>,
    pub timeout: Option<i32>,
    pub snapstart: bool,
    pub provisioned_concurrency: Option<i32>,
    pub reserved_concurrency: Option<i32>,
    pub enable_fs: bool,
    pub network: Option<Network>,
    pub fs: Option<FileSystem>,
    pub role: Role,
}

fn needs_fs(maybe_assets: Option<AssetsSpec>, mount_fs: Option<bool>) -> bool {
    if let Some(assets) = maybe_assets {
        let ax = assets.deps_path;
        match ax {
            Some(_) => true,
            None => match mount_fs {
                Some(f) => f,
                None => match assets.model_path {
                    Some(_) => true,
                    None => false,
                },
            },
        }
    } else {
        false
    }
}

fn make_network(infra_spec: &InfraSpec, enable_fs: bool) -> Option<Network> {
    if enable_fs {
        match &infra_spec.network {
            Some(net) => Some(Network {
                subnets: net.subnets.clone(),
                security_groups: net.security_groups.clone(),
            }),
            None => None,
        }
    } else {
        None
    }
}

fn make_fs(infra_spec: &InfraSpec, enable_fs: bool) -> Option<FileSystem> {
    if enable_fs {
        match &infra_spec.filesystem {
            Some(fs) => Some(FileSystem {
                arn: fs.arn.clone(),
                mount_point: fs.mount_point.clone(),
            }),
            None => None,
        }
    } else {
        None
    }
}

impl Runtime {
    pub fn new(
        fspec: &FunctionSpec
    ) -> Runtime {

        let rspec = safe_unwrap!("No runtime defined", fspec.runtime.clone());
        let role_spec = safe_unwrap!("No role_spec defined", rspec.role_spec);
        let infra_spec = rspec.infra_spec;
        let role = Role::new(&role_spec);

        let default_infra_spec = infra_spec.get("default").unwrap();

        let enable_fs = needs_fs(fspec.assets.clone(), rspec.mount_fs);

        Runtime {
            lang: rspec.lang.clone(),
            provider: Provider::Lambda,
            handler: rspec.handler,
            package_type: safe_unwrap!("No package_type defined", rspec.package_type),
            uri: safe_unwrap!("No uri defined", rspec.uri),
            layers: rspec.layers,
            environment: safe_unwrap!("No default vars", rspec.environment),
            provisioned_concurrency: rspec.provisioned_concurrency,
            reserved_concurrency: rspec.reserved_concurrency,
            memory_size: rspec.memory_size,
            timeout: rspec.timeout,
            cpu: None,
            snapstart: u::opt_as_bool(rspec.snapstart),
            role: role,
            enable_fs: false,
            fs: make_fs(&default_infra_spec, enable_fs),
            network: make_network(&default_infra_spec, enable_fs)
        }
    }
}
