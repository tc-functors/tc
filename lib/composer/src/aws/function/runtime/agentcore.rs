use super::common;
use compiler::{FunctionSpec, RuntimeSpec, InfraSpec, Arch};
use common::Runtime;
use common as c;
use std::collections::HashMap;

fn as_arch(maybe_arch: &Option<Arch>) -> Arch {
    match maybe_arch {
        Some(a) => a.clone(),
        None => Arch::X8664,
    }
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
    let uri = format!("{}/lambda.zip", dir);
    let enable_fs = false;
    let role = c::lookup_role(&infra_dir, &r, namespace, fqn, &fspec.name);
    let infra_spec = lookup_infraspec(infra_dir, &fspec.name, r);
    let default_infra_spec = infra_spec.get("default").unwrap();

    let InfraSpec {
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
        package_type: "zip".to_string(),
        uri: uri,
        layers: vec![],
        tags: c::make_tags(namespace, &infra_dir),
        environment: vars,
        provisioned_concurrency: None,
        reserved_concurrency: None,
        memory_size: None,
        timeout: *timeout,
        cpu: None,
        snapstart: false,
        role: role,
        enable_network: false,
        enable_fs: enable_fs,
        network: None,
        fs: None,
        arch: as_arch(&r.arch),
        infra_spec: infra_spec,
        microvm: None,
        port: 0
    }
}
