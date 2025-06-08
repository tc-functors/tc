use super::{layer, role};
use crate::spec::function::{
    AssetsSpec,
    BuildKind,
    FunctionSpec,
    Lang,
    Provider,
    LangRuntime,
    RuntimeSpec,
};
use crate::spec::infra::InfraSpec;
use crate::spec::ConfigSpec;

use crate::topology::{
    role::{
        Role,
    },
    template,
    version,
};


use chksum::sha1;
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashMap,
    fs::read_dir,
};

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
    pub tags: HashMap<String, String>,
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
    pub infra_spec: HashMap<String, InfraSpec>,
    pub cluster: String
}

fn find_code_version(dir: &str) -> String {
    let readdir = read_dir(dir).unwrap();
    let digest = sha1::chksum(readdir).unwrap();
    digest.to_hex_lowercase()
}

fn as_uri(dir: &str, name: &str, package_type: &str, uri: Option<String>) -> String {
    match package_type {
        "Image" | "image" | "oci" => match uri {
            Some(u) => u,
            None => {
                let version = find_code_version(dir);
                format!("{{{{repo}}}}/code:{}-{}", name, version)
            }
        },
        _ => format!("{}/lambda.zip", dir),
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

pub fn infer_lang(dir: &str) -> LangRuntime {
    if u::path_exists(dir, "handler.py") || u::path_exists(dir, "pyproject.toml") {
        LangRuntime::Python310
    } else if u::path_exists(dir, "Cargo.toml") {
        LangRuntime::Rust
    } else if u::path_exists(dir, "handler.js") || u::path_exists(dir, "package.json") {
        LangRuntime::Node22
    } else if u::path_exists(dir, "Gemfile") || u::path_exists(dir, "handler.rb") {
        LangRuntime::Ruby32
    } else if u::path_exists(dir, "deps.edn") {
        LangRuntime::Java21
    } else {
        LangRuntime::Python310
    }
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    u::file_exists(function_file) && u::file_exists(topology_file)
}

fn find_build_kind(fspec: &FunctionSpec) -> BuildKind {
    match &fspec.build {
        Some(b) => b.kind.clone(),
        None => BuildKind::Code,
    }
}

fn find_implicit_layer_name(dir: &str, namespace: &str, fspec: &FunctionSpec) -> Option<String> {
    let given_fqn = &fspec.fqn;
    let given_layer_name = &fspec.layer_name;

    let build_kind = find_build_kind(fspec);
    match given_layer_name {
        Some(name) => Some(name.to_string()),
        None => match build_kind {
            BuildKind::Code => {
                let lang = infer_lang(dir);
                if lang == LangRuntime::Ruby32 && layer::layerable(dir) {
                    match given_fqn {
                        Some(f) => Some(u::kebab_case(&f)),
                        None => {
                            if is_singular_function_dir() {
                                Some(s!(namespace))
                            } else {
                                Some(format!("{}-{}", namespace, &fspec.name))
                            }
                        }
                    }
                } else {
                    None
                }
            }
            _ => None,
        },
    }
}

fn follow_path(path: &str) -> String {
    if path.starts_with("..") {
        u::absolutize(&u::pwd(), path)
    } else {
        s!(path)
    }
}

fn as_infra_dir(dir: &str, _infra_dir: &str) -> String {
    let basename = u::basedir(dir).to_string();
    let parent = u::split_first(dir, &format!("/{basename}"));
    parent
        .replace("/services/", "/infrastructure/tc/")
        .replace("_", "-")
}

fn as_infra_spec_file(
    infra_dir: &str,
    rspec: &RuntimeSpec,
    function_name: &str,
) -> Option<String> {
    let f = format!("{}/vars/{}.json", infra_dir, function_name);
    let actual_f = follow_path(&f);
    if u::file_exists(&actual_f) {
        Some(actual_f)
    } else {
        match &rspec.vars_file {
            Some(p) => Some(follow_path(&p)),
            None => None,
        }
    }
}

fn lookup_role(
    infra_dir: &str,
    r: &RuntimeSpec,
    namespace: &str,
    function_name: &str,
) -> Role {

    match &r.role {
        Some(given) => role::use_given(&given),
        None => {
            let path = match &r.role_file {
                Some(f) => Some(follow_path(&f)),
                None => {
                    let f = format!("{}/roles/{}.json", infra_dir, function_name);
                    if u::file_exists(&f) { Some(f) } else { None }
                }
            };

            if let Some(p) = path {
                let abbr = if function_name.chars().count() > 15 {
                    u::abbreviate(function_name, "-")
                } else {
                    function_name.to_string()
                };
                let policy_name = format!("tc-{}-{{{{sandbox}}}}-{}-policy", namespace, abbr);
                let role_name = format!("tc-{}-{{{{sandbox}}}}-{}-role", namespace, abbr);

                role::make(&p, &role_name, &policy_name)
            } else {
                role::default(r.provider.clone())
            }


        }
    }
}


fn as_str(v: Option<String>, default: &str) -> String {
    match v {
        Some(s) => s.to_string(),
        None => String::from(default),
    }
}

fn make_env_vars(
    dir: &str,
    namespace: &str,
    build_kind: BuildKind,
    maybe_assets: Option<AssetsSpec>,
    environment: Option<HashMap<String, String>>,
    lang: Lang,
    fqn: &str,
) -> HashMap<String, String> {
    let mut hmap: HashMap<String, String> = HashMap::new();

    let mn = u::pascal_case(&format!("{} {}", namespace, fqn));

    hmap.insert(String::from("LAMBDA_STAGE"), template::profile());
    hmap.insert(String::from("Environment"), template::profile());
    hmap.insert(String::from("AWS_ACCOUNT_ID"), template::account());
    hmap.insert(String::from("SANDBOX"), template::sandbox());
    hmap.insert(String::from("NAMESPACE"), s!(namespace));
    hmap.insert(String::from("LOG_LEVEL"), s!("INFO"));
    hmap.insert(String::from("POWERTOOLS_METRICS_NAMESPACE"), mn);

    match lang {
        Lang::Ruby => match build_kind {
            BuildKind::Inline => {
                hmap.insert(s!("GEM_PATH"), s!("/var/task/gems/3.2.0"));
                hmap.insert(s!("GEM_HOME"), s!("/var/task/gems/3.2.0"));
                hmap.insert(s!("BUNDLE_CACHE_PATH"), s!("/var/task/vendor/cache"));
                hmap.insert(s!("RUBYLIB"), s!("$RUBYLIB:/var/task/lib"));
                hmap.insert(s!("LD_LIBRARY_PATH"),
                            s!("/var/lang/lib:/lib64:/usr/lib64:/var/runtime:/var/runtime/lib:/var/task:/var/task/lib"));
            }

            _ => {
                hmap.insert(s!("GEM_PATH"), s!("/opt/ruby/gems/3.2.0"));
                hmap.insert(s!("GEM_HOME"), s!("/opt/ruby/gems/3.2.0"));
                hmap.insert(s!("BUNDLE_CACHE_PATH"), s!("/opt/ruby/lib"));
                hmap.insert(s!("RUBYLIB"), s!("$RUBYLIB:/opt/lib"));

                if u::path_exists(dir, "Gemfile") {
                    hmap.insert(s!("AWS_LAMBDA_EXEC_WRAPPER"), s!("/opt/ruby/wrapper"));
                }
            }
        },
        Lang::Python => {
            // legacy
            if let Some(assets) = maybe_assets {
                let base_deps_path = as_str(assets.base_deps_path, "/var/python");
                let deps_path = as_str(assets.deps_path, "/var/python");
                let model_path = as_str(assets.model_path, "/model");

                hmap.insert(
                    s!("PYTHONPATH"),
                    format!(
                        "/opt/python:/var/runtime:{}/python:{}/python:{}",
                        &base_deps_path, &deps_path, &model_path
                    ),
                );
                hmap.insert(
                    s!("PATH"),
                    format!(
                        "/opt/python:/var/runtime:/model/bin:{}/python:{}/python:{}",
                        &base_deps_path, &deps_path, &model_path
                    ),
                );
                hmap.insert(s!("LD_LIBRARY_PATH"),
                            format!("/var/lang/lib:/lib64:/usr/lib64:/var/runtime:/var/runtime/lib:/var/task:/var/task/lib:/opt/lib:{}/lib:/model/lib", &deps_path));

                hmap.insert(s!("MODEL_PATH"), model_path);
                hmap.insert(s!("DEPS_PATH"), deps_path);
                hmap.insert(s!("BASE_DEPS_PATH"), base_deps_path);
            }
        },
        Lang::Node => match build_kind {
            BuildKind::Inline => {
                hmap.insert(s!("NODE_PATH"), s!("/var/task/node_modules"));
            },
            _ => (),
        },
        _ => (),
    }

    match environment {
        Some(e) => {
            hmap.extend(e);
            hmap
        }
        None => hmap,
    }
}

fn parent_tags_file(dir: &str) -> Option<String> {
    let paths = vec![
        u::absolutize(dir, "../tags.json"),
        u::absolutize(dir, "../../tags.json"),
        u::absolutize(dir, "../../../tags.json"),
        u::absolutize(dir, "../../../../tags.json"),
        s!("../tags.json"),
        s!("../../tags.json"),
        s!("../../../tags.json"),
        s!("../../../../tags.json"),
    ];
    u::any_path(paths)
}

fn load_tags(infra_dir: &str) -> HashMap<String, String> {
    let tags_file = format!("{}/tags.json", infra_dir);
    let parent_file = parent_tags_file(infra_dir);
    if u::file_exists(&tags_file) {
        let data: String = u::slurp(&tags_file);
        let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
        tags
    } else {
        match parent_file {
            Some(f) => {
                let data: String = u::slurp(&f);
                let tags: HashMap<String, String> = serde_json::from_str(&data).unwrap();
                tags
            }
            None => HashMap::new(),
        }
    }
}

fn make_tags(namespace: &str, infra_dir: &str) -> HashMap<String, String> {
    let tc_version = option_env!("PROJECT_VERSION")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();
    let version = version::current_semver(namespace);
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(s!("namespace"), s!(namespace));
    h.insert(s!("sandbox"), template::sandbox());
    h.insert(s!("version"), version);
    h.insert(s!("deployer"), s!("tc"));
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("tc_version"), tc_version);
    let given_tags = load_tags(infra_dir);
    h.extend(given_tags);
    h
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

fn lookup_infraspec(infra_dir: &str, name: &str, rspec: &RuntimeSpec) -> HashMap<String, InfraSpec> {
    let infra_spec_file = as_infra_spec_file(&infra_dir, rspec, name);
    InfraSpec::new(infra_spec_file.clone())
}

fn make_default(
    dir: &str,
    infra_dir: &str,
    namespace: &str,
    fqn: &str,
    fspec: &FunctionSpec
) -> Runtime {
    let lang = infer_lang(dir);
    let role = role::default(Some(Provider::Lambda));
    let infra_spec = InfraSpec::new(None);
    let default_infra_spec = infra_spec.get("default").unwrap();

    let InfraSpec {
        memory_size,
        timeout,
        environment,
        ..
    } = default_infra_spec;


    let vars = make_env_vars(
        dir,
        namespace,
        BuildKind::Code,
        fspec.assets.clone(),
        environment.clone(),
        lang.to_lang(),
        fqn,
    );

     Runtime {
         lang: lang,
         provider: Provider::Lambda,
         handler: s!("handler.handler"),
         package_type: s!("zip"),
         uri: as_uri(dir, &fspec.name, "zip", None),
         layers: vec![],
         environment: vars,
         tags: make_tags(namespace, &infra_dir),
         provisioned_concurrency: default_infra_spec.provisioned_concurrency.clone(),
         reserved_concurrency: default_infra_spec.reserved_concurrency.clone(),
         role: role,
         memory_size: *memory_size,
         cpu: None,
         timeout: *timeout,
         snapstart: false,
         enable_fs: false,
         network: None,
         fs: None,
         infra_spec: infra_spec,
         cluster: String::from("")
     }
}

fn make_lambda(
    dir: &str,
    infra_dir: &str,
    namespace: &str,
    fqn: &str,
    fspec: &FunctionSpec,
    r: &RuntimeSpec
) -> Runtime {

    let layer_name = find_implicit_layer_name(dir, namespace, fspec);
    let layers = consolidate_layers(r.extensions.clone(), r.layers.clone(), layer_name);
    let package_type = &r.package_type;
    let uri = as_uri(dir, &fspec.name, package_type, r.uri.clone());
    let enable_fs = needs_fs(fspec.assets.clone(), r.mount_fs);
    let role = lookup_role(&infra_dir, &r, namespace, &fspec.name);

    let build_kind = find_build_kind(&fspec);
    let infra_spec = lookup_infraspec(infra_dir, &fspec.name, r);
    let default_infra_spec = infra_spec.get("default").unwrap();

    let InfraSpec {
        memory_size,
        timeout,
        environment,
        ..
    } = default_infra_spec;

    let vars = make_env_vars(
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
        tags: make_tags(namespace, &infra_dir),
        environment: vars,
        provisioned_concurrency: default_infra_spec.provisioned_concurrency.clone(),
        reserved_concurrency: default_infra_spec.reserved_concurrency.clone(),
        memory_size: *memory_size,
        timeout: *timeout,
        cpu: None,
        snapstart: u::opt_as_bool(r.snapstart),
        role: role,
        enable_fs: enable_fs,
        network: make_network(&default_infra_spec, enable_fs),
        fs: make_fs(&default_infra_spec, enable_fs),
        infra_spec: infra_spec,
        cluster: String::from("")
    }
}

fn make_fargate(
    dir: &str,
    infra_dir: &str,
    namespace: &str,
    fqn: &str,
    fspec: &FunctionSpec,
    rspec: &RuntimeSpec,
    c: &ConfigSpec
) -> Runtime {
    let enable_fs = needs_fs(fspec.assets.clone(), rspec.mount_fs);
    let package_type = s!("Image");
    let uri = as_uri(dir, &fspec.name, &package_type, rspec.uri.clone());
    let role = lookup_role(&infra_dir, &rspec, namespace, &fspec.name);
    let infra_spec = lookup_infraspec(infra_dir, &fspec.name, &rspec);
    let default_infra_spec = infra_spec.get("default").unwrap();

    let lang = rspec.lang.clone();
    let cluster = match &c.builder.cluster {
        Some(c) => c,
        None => &template::topology_fqn(namespace, false)
    };

    let InfraSpec {
        timeout,
        environment,
        ..
    } = default_infra_spec;


    let vars = make_env_vars(
        dir,
        namespace,
        BuildKind::Code,
        fspec.assets.clone(),
        environment.clone(),
        lang.to_lang(),
        fqn,
    );

    Runtime {
        lang: lang,
        provider: Provider::Fargate,
        handler: rspec.handler.clone(),
        package_type: package_type,
        uri: uri,
        layers: vec![],
        tags: make_tags(namespace, &infra_dir),
        environment: vars,
        provisioned_concurrency: None,
        reserved_concurrency: None,
        memory_size: Some(2048),
        cpu: Some(1024),
        timeout: *timeout,
        snapstart: false,
        role: role,
        enable_fs: enable_fs,
        network: make_network(&default_infra_spec, enable_fs),
        fs: make_fs(&default_infra_spec, enable_fs),
        infra_spec: infra_spec,
        cluster: cluster.to_string()
    }
}

impl Runtime {

    pub fn new(
        dir: &str,
        t_infra_dir: &str,
        namespace: &str,
        fspec: &FunctionSpec,
        fqn: &str,
        cspec: &ConfigSpec
    ) -> Runtime {

        let rspec = fspec.runtime.clone();

        let infra_dir = match &fspec.infra_dir {
            Some(p) => p.to_string(),
            None => as_infra_dir(dir, t_infra_dir),
        };

        match rspec {
            Some(r) => {
                if let Some(ref provider) = r.provider {
                    match provider {
                        Provider::Lambda =>
                            make_lambda(
                                dir,
                                &infra_dir,
                                namespace,
                                fqn,
                                fspec,
                                &r
                            ),

                        Provider::Fargate =>
                            make_fargate(
                                dir,
                                &infra_dir,
                                namespace,
                                fqn,
                                fspec,
                                &r,
                                cspec
                            )
                    }
                } else {
                    make_lambda(
                        dir,
                        &infra_dir,
                        namespace,
                        fqn,
                        fspec,
                        &r
                    )
                }

            }
            None => make_default(
                dir,
                &infra_dir,
                namespace,
                fqn,
                fspec
            )
        }
    }
}
