use super::layer;
use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn default_lang() -> String {
    s!("python3.10")
}

fn default_handler() -> String {
    s!("handler.handler")
}

fn default_layers() -> Vec<String> {
    vec![]
}

fn default_version() -> String {
    s!("0.0.1")
}

fn default_fqn() -> String {
    u::empty()
}

fn default_namespace() -> String {
    u::empty()
}

fn default_infra_dir() -> String {
    u::empty()
}

fn default_layer_name() -> Option<String> {
    None
}
fn default_package_type() -> String {
    s!("zip")
}

fn default_image_uri() -> String {
    s!("public.ecr.aws/sam/build-python3.10:latest")
}

fn default_role() -> Role {
    Role {
        name: s!("default"),
        path: s!("tc-base-lambda-role"),
        policy_name: s!("tc-base-lambda-policy"),
    }
}

fn default_container_task() -> ContainerTask {
    ContainerTask::new("default")
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerTask {
    pub name: String,
    pub task_role: String,
    pub image_uri: String,
    pub command: String
}

impl ContainerTask {
    pub fn new(name: &str) -> ContainerTask {
        ContainerTask {
            name: s!(name),
            task_role: s!("tc-base-task-role"),
            image_uri: s!("public.ecr.aws/docker/library/python:3.11"),
            command: s!("python /app/run.py"),
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RuntimeSpec {
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default = "default_handler")]
    pub handler: String,
    #[serde(default = "default_package_type")]
    pub package_type: String,
    #[serde(default = "default_image_uri")]
    pub image_uri: String,
    #[serde(default = "default_layers")]
    pub layers: Vec<String>,
    #[serde(default = "default_layers")]
    pub extensions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Role {
    pub name: String,
    pub path: String,
    pub policy_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionSpec {
    pub name: String,
    #[serde(default = "default_fqn")]
    pub namespace: String,
    #[serde(default = "default_namespace")]
    pub fqn: String,
    #[serde(default = "default_layer_name")]
    pub layer_name: Option<String>,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_version")]
    pub revision: String,
    #[serde(default)]
    pub dir: String,
    #[serde(default = "default_infra_dir")]
    pub infra_dir: String,
    #[serde(default)]
    pub description: Option<String>,
    pub runtime: RuntimeSpec,
    #[serde(default)]
    pub tasks: HashMap<String, String>,
    #[serde(default)]
    pub assets: HashMap<String, Value>,
    #[serde(default)]
    pub vars_file: Option<String>,
    #[serde(default = "default_role")]
    pub role: Role,
    #[serde(default = "default_container_task")]
    pub container_task: ContainerTask
}

fn as_infra_dir(dir: &str, infra_dir: &str) -> String {
    if infra_dir.is_empty() {
        let basename = u::basedir(dir).to_string();
        let parent = u::split_first(dir, &format!("/{basename}"));
        parent
            .replace("/services/", "/infrastructure/tc/")
            .replace("_", "-")
    } else {
        if infra_dir.starts_with("..") {
            u::absolutize(&u::pwd(), infra_dir)
        } else {
            s!(infra_dir)
        }
    }
}

pub fn read_pyp(path: &str) -> Pyp {
    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let value: Pyp = toml::from_str(&data).unwrap();
    value
}

#[derive(Deserialize)]
pub struct Poetry {
    pub version: Option<String>,
}

#[derive(Deserialize)]
pub struct Tool {
    pub poetry: Poetry,
}

#[derive(Deserialize)]
pub struct Pyp {
    pub tool: Tool,
}

fn find_version(dir: &str) -> String {
    let pyp = &format!("{}/pyproject.toml", dir);
    if u::file_exists(pyp) {
        let table = read_pyp(pyp);
        let version = table.tool.poetry.version;
        match version {
            Some(v) => v,
            None => s!("0.0.1"),
        }
    } else {
        s!("0.0.1")
    }
}

fn find_revision(dir: &str) -> String {
    let cmd_str = format!("git log -n 1 --format=%h {}", dir);
    u::sh(&cmd_str, dir)
}

fn render(s: &str, version: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("version", version);
    table.insert("sandbox", "{{sandbox}}");
    u::stencil(s, table)
}

pub fn infer_lang(dir: &str) -> &str {
    if u::path_exists(dir, "handler.py") || u::path_exists(dir, "pyproject.toml") {
        "python3.10"
    } else if u::path_exists(dir, "project.janet") {
        "janet"
    } else if u::path_exists(dir, "Cargo.toml") {
        "rust"
    } else if u::path_exists(dir, "Gemfile") {
        "ruby3.2"
    } else if u::path_exists(dir, "handler.rb") {
        "ruby3.2"
    } else if u::path_exists(dir, "deps.edn") {
        "java"
    } else if u::path_exists(dir, "main.go") || u::path_exists(dir, "go.mod") {
        "go"
    } else {
        "default"
    }
}

pub fn is_function_dir(path: &str) -> bool {
    u::path_exists(path, "function.json")
}

pub fn is_inferable(dir: &str) -> bool {
    let lang = infer_lang(dir);
    lang != "default"
}

fn find_layer_name(dir: &str, namespace: &str, fspec: &FunctionSpec) -> Option<String> {
    let given_fqn = &fspec.fqn;
    let given_layer_name = &fspec.layer_name;

    match given_layer_name {
        Some(name) => Some(name.to_string()),
        None => {
            let lang = infer_lang(dir);
            if lang == "ruby3.2" && layer::layerable(dir) {
                if given_fqn.is_empty() {
                    if is_singular_function_dir() {
                        Some(s!(namespace))
                    } else {
                        Some(format!("{}-{}", namespace, &fspec.name))
                    }
                } else {
                    Some(u::kebab_case(&given_fqn))
                }
            } else {
                None
            }
        }
    }
}

fn maybe_vars_file(infra_dir: &str, function_name: &str) -> Option<String> {
    let f = format!("{}/vars/{}.json", infra_dir, function_name);
    if u::file_exists(&f) {
        Some(f)
    } else {
        None
    }
}

fn role_of(namespace: &str, roles_file: &str, function_name: &str) -> Role {
    let abbr = if function_name.chars().count() > 15 {
        u::abbreviate(function_name, "-")
    } else {
        function_name.to_string()
    };
    if u::file_exists(roles_file) {
        Role {
            name: format!("tc-{}-{{{{sandbox}}}}-{}-role", namespace, abbr),
            path: roles_file.to_string(),
            policy_name: format!("tc-{}-{{{{sandbox}}}}-{}-policy", namespace, abbr),
        }
    } else {
        Role {
            name: s!("tc-base-lambda-role"),
            path: s!("tc-base-lambda-role"),
            policy_name: s!("tc-base-lambda-policy"),
        }
    }
}

fn is_singular_function_dir() -> bool {
    let function_file = "function.json";
    let topology_file = "topology.yml";
    u::file_exists(function_file) && u::file_exists(topology_file)
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
            _ => format!("{}_{}_{{{{sandbox}}}}", namespace, name),
        }
    }
}

fn consolidate_layers(
    extensions: &mut Vec<String>,
    given_layers: &mut Vec<String>,
    implicit_layer: Option<String>,
) -> Vec<String> {
    let mut layers: Vec<String> = vec![];
    layers.append(given_layers);
    layers.append(extensions);
    match implicit_layer {
        Some(m) => layers.push(m),
        None => (),
    }
    u::uniq(layers)
}

fn find_roles_file(infra_dir: &str, alt_infra_dir: &str, function_name: &str) -> String {
    let f = format!("{}/roles/{}.json", infra_dir, function_name);
    let alt_f = format!("{}/roles/{}.json", alt_infra_dir, function_name);
    if u::file_exists(&f) {
        f
    } else if u::file_exists(&alt_f) {
        alt_f
    } else {
        u::empty()
    }
}

// parse partial function.json
fn load(dir: &str, infra_dir: &str, namespace: &str, format: &str) -> FunctionSpec {
    let f = format!("{}/function.json", dir);

    let version = find_version(dir);
    let revision = find_revision(dir);
    let data = render(&u::slurp(&f), &version);
    let mut f: FunctionSpec = serde_json::from_str(&data).unwrap();
    f.dir = dir.to_string();

    let layer_name = find_layer_name(dir, namespace, &f);
    f.layer_name = layer_name.to_owned();
    f.runtime.layers =
        consolidate_layers(&mut f.runtime.extensions, &mut f.runtime.layers, layer_name);

    let another_infra_dir = as_infra_dir(dir, &f.infra_dir);
    let vars_file = match maybe_vars_file(infra_dir, &f.name) {
        Some(p) => Some(p),
        None => maybe_vars_file(&another_infra_dir, &f.name),
    };

    let roles_file = find_roles_file(infra_dir, &another_infra_dir, &f.name);

    f.namespace = namespace.to_owned();
    f.version = version;
    f.revision = revision;
    f.vars_file = vars_file;
    f.role = if &f.role.name == "default" {
        role_of(namespace, &roles_file, &f.name)
    } else {
        f.role
    };
    f.infra_dir = infra_dir.to_string();
    f.fqn = find_fqn(&f.fqn, namespace, &f.name, format);
    f
}

// parse no function.json
fn infer(dir: &str, infra_dir: &str, namespace: &str) -> FunctionSpec {
    let fn_name = u::basedir(dir).to_string();
    let another_infra_dir = as_infra_dir(dir, "");
    let lang = infer_lang(dir);
    let layers = vec![];

    let runtime = RuntimeSpec {
        lang: lang.to_string(),
        handler: s!("handler.handler"),
        package_type: s!("zip"),
        image_uri: default_image_uri(),
        layers: layers,
        extensions: vec![],
    };

    let mut tasks: HashMap<String, String> = HashMap::new();

    let build_task = match lang {
        "python3.7"  => s!("zip -r -q lambda.zip *.py"),
        "python3.9"  => s!("zip -r -q lambda.zip *.py"),
        "python3.10" => s!("zip -r -q lambda.zip *.py"),
        "python3.11" => s!("zip -r -q lambda.zip *.py"),
        "python3.12" => s!("zip -r -q lambda.zip *.py"),
        "ruby3.2"    => s!("zip -r -q lambda.zip *.rb"),
        "janet"      => s!("zip -r -q lambda.zip *.janet"),
        _            => s!("zip -r -q lambda.zip bootstrap"),
    };
    tasks.insert(s!("build"), build_task);
    tasks.insert(s!("clean"), s!("rm -f lambda.zip"));

    let fqn = if namespace.is_empty() {
        format!("{}_{{{{sandbox}}}}", &fn_name)
    } else {
        format!("{}_{}_{{{{sandbox}}}}", namespace, &fn_name)
    };

    let vars_file = maybe_vars_file(&infra_dir, &fn_name);
    let roles_file = find_roles_file(infra_dir, &another_infra_dir, &fn_name);
    FunctionSpec {
        name: fn_name.to_owned(),
        namespace: namespace.to_string(),
        dir: s!(dir),
        infra_dir: infra_dir.to_owned(),
        fqn: fqn,
        layer_name: None,
        version: s!("0.0.1"),
        revision: s!("0.0.1"),
        description: None,
        runtime: runtime,
        tasks: tasks,
        assets: HashMap::new(),
        vars_file: vars_file,
        role: role_of(namespace, &roles_file, &fn_name),
        container_task: ContainerTask::new(&fn_name)
    }
}

impl FunctionSpec {
    pub fn new(dir: &str, infra_dir: &str, namespace: &str, format: &str) -> Option<FunctionSpec> {
        if is_function_dir(dir) {
            Some(load(dir, infra_dir, namespace, format))
        } else if is_inferable(dir) {
            Some(infer(dir, infra_dir, namespace))
        } else {
            None
        }
    }

    pub fn parse(dir: &str) -> FunctionSpec {
        let f = format!("{}/function.json", dir);
        let data = u::slurp(&f);
        let fspec: FunctionSpec = serde_json::from_str(&data).unwrap();
        fspec
    }

    pub fn to_map(function: Option<FunctionSpec>) -> HashMap<String, FunctionSpec> {
        let mut fns: HashMap<String, FunctionSpec> = HashMap::new();
        match function {
            Some(f) => fns.insert(f.dir.to_string(), f),
            None => None,
        };
        fns
    }
}
