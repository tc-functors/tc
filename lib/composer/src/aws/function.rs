pub mod build;
pub mod code;
pub mod layer;
pub mod runtime;
pub mod target;

use super::template;
pub use build::Build;
use compiler::spec::{
    TestSpec,
    function::FunctionSpec,
};
use configurator::Config;
use kit as u;
use kit::*;
use runtime::collect_aux_files;
pub use runtime::Runtime;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use target::Target;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
    pub name: String,
    pub actual_name: String,
    pub namespace: String,
    pub dir: String,
    pub description: Option<String>,
    pub fqn: String,
    pub arn: String,
    pub layer_name: Option<String>,
    pub version: String,
    pub runtime: Runtime,
    pub build: Build,
    pub test: HashMap<String, TestSpec>,
    pub targets: Vec<Target>,
    /// Marks functions imported via relative `uri:` for dedup promotion to root.
    #[serde(default)]
    pub shared: bool,
    /// Absolute paths of files outside `f.dir` whose contents the
    /// composer read while constructing this function: role JSON,
    /// vars JSON, inherited parent `roles/function.json`, etc. Used
    /// by the differ to widen the per-function closure beyond the
    /// source-code closure — a change to any of these files marks the
    /// function dirty even when no source code changed.
    ///
    /// Conventional paths (`{infra_dir}/roles/{name}.json`,
    /// `{infra_dir}/vars/{name}.json`) are always present, even when
    /// the file doesn't currently exist on disk, so deletions
    /// (visible in `git diff` as removed paths) still flip the
    /// function dirty. Explicit overrides (`r.role_file`,
    /// `r.vars_file`) are present only when they were actually used
    /// by the composer.
    ///
    /// Lives on `Function` rather than on `Runtime` because these are
    /// source-tree paths on the developer's machine — the deployed
    /// Lambda runtime never sees them. They're a sibling of `dir`,
    /// not part of the runtime config.
    ///
    /// Populated by `collect_aux_files` during compose. Defaults to an
    /// empty `Vec` so resolver-cached topologies pre-dating this field
    /// continue to deserialize cleanly (they degrade to today's
    /// behavior for that one cache hit; the next compose repopulates).
    #[serde(default)]
    pub aux_files: Vec<String>,
}

fn is_singular_function_dir() -> bool {
    let function_file = &compiler::spec::function::find_fspec_file(&u::pwd());
    let function_file_json = "function.json";
    let topology_file = "topology.yml";
    (u::file_exists(function_file) || u::file_exists(function_file_json))
        && u::file_exists(topology_file)
}

fn find_fqn(given_fqn: &str, namespace: &str, name: &str, format: &str) -> String {
    if !given_fqn.is_empty() {
        format!("{}_{{{{sandbox}}}}", given_fqn)
    } else if !name.is_empty() && namespace.is_empty() {
        format!("{}_{{{{sandbox}}}}", name)
    } else if is_singular_function_dir() {
        format!("{}_{{{{sandbox}}}}", namespace)
    } else {
        match format {
            "hyphenated" => format!("{}-{}-{{{{sandbox}}}}", namespace, name),
            _ => {
                if namespace.is_empty() {
                    format!("{}_{{{{sandbox}}}}", name)
                } else {
                    format!("{}_{}_{{{{sandbox}}}}", namespace, name)
                }
            }
        }
    }
}

fn make_fqn(fspec: &FunctionSpec, namespace: &str, format: &str) -> String {
    match &fspec.fqn {
        Some(f) => find_fqn(&f, namespace, &fspec.name, format),
        None => match &fspec.namespace {
            Some(n) => {
                if format == "hyphenated" {
                    format!("{}-{}-{{{{sandbox}}}}", n, &fspec.name)
                } else {
                    format!("{}_{}_{{{{sandbox}}}}", n, &fspec.name)
                }
            }
            None => {
                if format == "hyphenated" {
                    format!("{}-{}-{{{{sandbox}}}}", namespace, &fspec.name)
                } else {
                    if namespace.is_empty() {
                        format!("{}_{{{{sandbox}}}}", &fspec.name)
                    } else {
                        format!("{}_{}_{{{{sandbox}}}}", namespace, &fspec.name)
                    }
                }
            }
        },
    }
}

fn make_test(t: Option<HashMap<String, TestSpec>>) -> HashMap<String, TestSpec> {
    match t {
        Some(spec) => spec,
        None => HashMap::new(),
    }
}

impl Function {
    pub fn new(dir: &str, topo_infra_dir: &str, namespace: &str, format: &str) -> Function {
        let config = Config::new();

        let fspec = FunctionSpec::new(dir);

        let namespace = match fspec.namespace {
            Some(ref n) => n,
            None => &namespace.to_string(),
        };
        let fqn = make_fqn(&fspec, &namespace, format);

        let infra_dir = match fspec.infra_dir {
            Some(ref d) => &d,
            None => topo_infra_dir,
        };

        let runtime = Runtime::new(dir, infra_dir, &namespace, &fspec, &fqn, &config);
        let mut aux_files = collect_aux_files(
            infra_dir,
            &fspec,
            fspec.runtime.as_ref(),
            &runtime.role,
        );

        if let Some(ref afiles) =  fspec.aux_files {
            aux_files.extend(afiles.clone());
        }

        let targets = Target::make_all(&fspec);

        Function {
            name: fspec.name.to_string(),
            actual_name: fspec.name.to_string(),
            arn: template::lambda_arn(&fqn),
            version: s!(""),
            fqn: fqn.clone(),
            description: None,
            dir: dir.to_string(),
            namespace: namespace.to_string(),
            build: Build::new(dir, &runtime, fspec.build, fspec.tasks),
            layer_name: fspec.layer_name,
            test: make_test(fspec.test),
            runtime: runtime,
            targets: targets,
            shared: false,
            aux_files: aux_files,
        }
    }

    pub fn from_spec(
        fspec: &FunctionSpec,
        namespace: &str,
        dir: &str,
        infra_dir: &str,
    ) -> Function {
        let config = Config::new();
        let namespace = match fspec.namespace {
            Some(ref n) => n,
            None => &namespace.to_string(),
        };
        let fqn = make_fqn(&fspec, &namespace, "");

        let infra_dir = match fspec.infra_dir {
            Some(ref d) => &d,
            None => infra_dir,
        };

        let runtime = Runtime::new(dir, infra_dir, &namespace, &fspec, &fqn, &config);
        let aux_files = collect_aux_files(
            infra_dir,
            fspec,
            fspec.runtime.as_ref(),
            &runtime.role,
        );

        let targets = Target::make_all(&fspec);

        Function {
            name: fspec.name.to_string(),
            actual_name: fspec.name.to_string(),
            arn: template::lambda_arn(&fqn),
            version: s!(""),
            fqn: fqn.clone(),
            description: None,
            dir: dir.to_string(),
            namespace: namespace.to_string(),
            build: Build::new(dir, &runtime, fspec.build.clone(), fspec.tasks.clone()),
            layer_name: fspec.layer_name.clone(),
            test: make_test(fspec.test.clone()),
            runtime: runtime,
            targets: targets,
            shared: false,
            aux_files: aux_files,
        }
    }

    pub fn to_map(function: Function) -> HashMap<String, Function> {
        let mut fns: HashMap<String, Function> = HashMap::new();
        fns.insert(function.name.to_string(), function);
        fns
    }
}
