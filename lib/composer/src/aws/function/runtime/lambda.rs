use super::{
    common,
    layer,
};
use common as c;
use common::{
    FileSystem,
    Network,
    Runtime,
};
use compiler::{
    Arch,
    BuildKind,
    FunctionSpec,
    InfraSpec,
    RuntimeSpec,
    function::{
        AssetsSpec,
        FileSystemKind,
        FileSystemSpec,
    },
};
use kit as u;
use kit::*;
use std::collections::HashMap;

fn find_image_tag(dir: &str, namespace: &str) -> String {
    match std::env::var("TC_VERSION_IMAGES") {
        Ok(_) => u::current_semver(namespace),
        Err(_) => c::find_git_sha(dir),
    }
}

fn consolidate_layers(
    extensions: Vec<String>,
    given_layers: Vec<String>,
    implicit_layer: Option<String>,
) -> Vec<String> {
    let mut layers: Vec<String> = vec![];
    let mut e: Vec<String> = extensions;
    let mut g: Vec<String> = given_layers;
    layers.append(&mut e);
    layers.append(&mut g);

    match implicit_layer {
        Some(m) => layers.push(m),
        None => (),
    }
    u::uniq(layers)
}

fn as_uri(
    dir: &str,
    namespace: &str,
    name: &str,
    package_type: &str,
    uri: Option<String>,
    build_kind: &BuildKind
) -> String {
    match package_type {
        "Image" | "image" | "oci" => match uri {
            Some(u) => u,
            None => {
                let tag = find_image_tag(dir, namespace);
                format!("{{{{repo}}}}:{}_{}_{}", namespace, name, &tag)
            }
        },
        _ => {
            match build_kind {
                BuildKind::Inline => {
                    match std::env::var("TC_USE_ASSET_STORE") {
                        Ok(_) => {
                            let key = format!("{}/{{{{version}}}}/functions/{}.zip",
                                              namespace, name);
                            format!("s3://{{{{ASSET_BUCKET}}}}/{}", &key)
                        }
                        Err(_) => format!("{}/lambda.zip", dir)
                    }
                },
                _ => format!("{}/lambda.zip", dir)

            }
        }
    }
}

fn needs_fs(
    maybe_assets: Option<AssetsSpec>,
    mount_fs: Option<bool>,
    fs: &Option<FileSystemSpec>,
) -> bool {
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
        match fs {
            Some(_) => true,
            None => false,
        }
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

fn as_fs_kind(fs_spec: &Option<FileSystemSpec>) -> FileSystemKind {
    match fs_spec {
        Some(f) => match &f.kind {
            Some(p) => p.clone(),
            None => FileSystemKind::Efs,
        },
        None => FileSystemKind::Efs,
    }
}

fn make_fs(
    infra_spec: &InfraSpec,
    fs_spec: &Option<FileSystemSpec>,
    enable_fs: bool,
) -> Option<FileSystem> {
    if enable_fs {
        match &infra_spec.filesystem {
            Some(fs) => Some(FileSystem {
                kind: as_fs_kind(fs_spec),
                arn: fs.arn.clone(),
                mount_point: fs.mount_point.clone(),
            }),
            None => None,
        }
    } else {
        None
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

fn as_arch(maybe_arch: &Option<Arch>) -> Arch {
    match maybe_arch {
        Some(a) => a.clone(),
        None => Arch::X8664,
    }
}

pub fn make(
    dir: &str,
    infra_dir: &str,
    namespace: &str,
    fqn: &str,
    fspec: &FunctionSpec,
    r: &RuntimeSpec,
) -> Runtime {
    let layer_name = layer::find_implicit_layer_name(dir, namespace, fspec);
    let layers = consolidate_layers(r.extensions.clone(), r.layers.clone(), layer_name);
    let build_kind = c::find_build_kind(&fspec);
    let package_type = match &r.package_type {
        Some(x) => x.to_string(),
        None => match build_kind {
            BuildKind::Image => s!("image"),
            _ => s!("zip"),
        },
    };
    let uri = as_uri(dir, namespace, &fspec.name, &package_type, r.uri.clone(), &build_kind);
    let enable_fs = needs_fs(fspec.assets.clone(), r.mount_fs, &r.fs);
    let role = c::lookup_role(&infra_dir, &r, namespace, fqn, &fspec.name);

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
        layers: layers,
        tags: c::make_tags(namespace, &infra_dir),
        environment: vars,
        provisioned_concurrency: default_infra_spec.provisioned_concurrency.clone(),
        reserved_concurrency: default_infra_spec.reserved_concurrency.clone(),
        memory_size: *memory_size,
        timeout: *timeout,
        cpu: None,
        snapstart: u::opt_as_bool(r.snapstart),
        role: role,
        enable_network: if let Some(n) = r.network { n } else { false },
        enable_fs: enable_fs,
        network: make_network(&default_infra_spec, enable_fs),
        fs: make_fs(&default_infra_spec, &r.fs, enable_fs),
        arch: as_arch(&r.arch),
        infra_spec: infra_spec,
        microvm: None,
        port: match r.port {
            Some(p) => p,
            None => 8080,
        },
    }
}
