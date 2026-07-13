use super::common;
use crate::{
    Role,
    index,
};
use common as c;
use common::Runtime;
use compiler::{
    Arch,
    BuildKind,
    Entity,
    FunctionSpec,
    InfraSpec,
    RuntimeSpec,
    function::MicroVm,
};
use kit as u;
use kit::*;
use std::collections::HashMap;

fn lookup_role(
    infra_dir: &str,
    r: &RuntimeSpec,
    namespace: &str,
    _fqn: &str,
    function_name: &str,
) -> Role {
    match &r.role {
        Some(given) => Role::provided(&given),
        None => {
            let f = format!("{}/roles/{}.json", infra_dir, function_name);
            if index::get().file_exists(&f) {
                Role::new(Entity::Function, &f, namespace, function_name)
            } else {
                Role::default_microvm()
            }
        }
    }
}

fn make_microvm() -> Option<MicroVm> {
    Some(MicroVm {
        ingress_network_connectors: Some(format!(
            "arn:aws:lambda:{{{{region}}}}:aws:network-connector:aws-network-connector:ALL_INGRESS"
        )),
        egress_network_connectors: Some(format!(
            "arn:aws:lambda:{{{{region}}}}:aws:network-connector:aws-network-connector:INTERNET_EGRESS"
        )),
        max_duration: Some(3600),
        log_group: None,
    })
}

fn lookup_infraspec(
    infra_dir: &str,
    name: &str,
    rspec: &RuntimeSpec,
) -> HashMap<String, InfraSpec> {
    let infra_spec_file = c::as_infra_spec_file(&infra_dir, rspec, name);
    InfraSpec::new(infra_spec_file.clone())
}

pub fn make(
    dir: &str,
    infra_dir: &str,
    namespace: &str,
    fqn: &str,
    fspec: &FunctionSpec,
    r: &RuntimeSpec,
) -> Runtime {
    let build_kind = c::find_build_kind(&fspec);
    let package_type = match &r.package_type {
        Some(x) => x.to_string(),
        None => match build_kind {
            BuildKind::Image => s!("image"),
            _ => s!("zip"),
        },
    };
    let uri = format!("{}/lambda.zip", dir);
    let enable_fs = false;
    let role = lookup_role(&infra_dir, &r, namespace, fqn, &fspec.name);

    let infra_spec = lookup_infraspec(infra_dir, &fspec.name, r);
    let default_infra_spec = infra_spec.get("default").unwrap();

    let InfraSpec {
        memory_size,
        timeout,
        environment,
        ..
    } = default_infra_spec;

    let vars = c::make_env_vars(
        dir,
        namespace,
        build_kind,
        fspec.assets.clone(),
        environment.clone(),
        r.lang.to_lang(),
        fqn,
    );

    Runtime {
        lang: r.lang.clone(),
        provider: r.provider.clone().unwrap().clone(),
        handler: r.handler.clone(),
        package_type: package_type.to_string(),
        uri: uri,
        layers: vec![],
        tags: c::make_tags(namespace, &infra_dir),
        environment: vars,
        provisioned_concurrency: default_infra_spec.provisioned_concurrency.clone(),
        reserved_concurrency: default_infra_spec.reserved_concurrency.clone(),
        memory_size: match r.mem {
            Some(m) => Some(m),
            None => *memory_size,
        },
        timeout: *timeout,
        cpu: None,
        snapstart: u::opt_as_bool(r.snapstart),
        role: role,
        enable_network: if let Some(n) = r.network { n } else { false },
        enable_fs: enable_fs,
        network: None,
        fs: None,
        arch: Arch::Arm64,
        infra_spec: infra_spec,
        microvm: make_microvm(),
        port: match r.port {
            Some(p) => p,
            None => 8080,
        },
    }
}
