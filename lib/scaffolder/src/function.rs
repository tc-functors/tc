use compiler::{
    Entity,
    spec::{
        Lang,
        function::{
            BuildKind,
            LangRuntime,
        },
    },
};
use composer::topology::Role;
use inquire::{
    Confirm,
    InquireError,
    Select,
    Text,
};
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::str::FromStr;

// handler

fn gen_py_handler() -> String {
    format!(
        r#"
def handler(event, context):
    return {{'status': 'ok'}}
"#
    )
}

fn gen_ruby_handler() -> String {
    format!(
        r#"
require 'json'

def handler(event:, context:)
    {{ event: JSON.generate(event), context: JSON.generate(context.inspect) }}
end
"#
    )
}

fn gen_node_handler() -> String {
    format!(
        r#"
const handler = async(event) => {{
    try {{
      return 'Success';
    }} catch (error) {{
        console.error(`Failed to process order: ${{error.message}}`);
        throw error;
    }}
}};

module.exports = {{
  handler
}};
"#
    )
}

pub fn write_handler(fdir: &str, langr: &LangRuntime) {
    let data = match langr.to_lang() {
        Lang::Python => gen_py_handler(),
        Lang::Ruby => gen_ruby_handler(),
        Lang::Node => gen_node_handler(),
        _ => String::from(""),
    };
    let file = match langr.to_lang() {
        Lang::Python => "handler.py",
        Lang::Ruby => "handler.rb",
        Lang::Node => "handler.js",
        _ => "handler.unknown",
    };
    let path = format!("{}/{}", fdir, &file);
    if !u::file_exists(&path) {
        u::write_str(&path, &data);
    }
}

// infra

fn default_vars(memory: &str, timeout: &str) -> String {
    format!(
        r#"{{
  "default": {{
    "timeout": {timeout},
    "memory_size": {memory},
    "environment": {{
      "LOG_LEVEL": "INFO"
    }}
  }},
  "dev": {{
    "default": {{
      "timeout": {timeout},
      "memory_size": {memory},
      "environment": {{
	"LOG_LEVEL": "INFO"
      }}
    }}
  }}
}}"#
    )
}

fn write_role(roles_dir: &str, name: &str, role: &Role) {
    let role_path = format!("{}/{}.json", roles_dir, name);
    if !u::file_exists(&role_path) {
        println!("Scaffolding role {}", &role_path);
        let data = serde_json::to_string_pretty(&role.policy).unwrap();
        u::write_str(&role_path, &data);
    } else {
        println!("roles for {} exists, skipping", name);
    }
}

fn write_vars(vars_dir: &str, name: &str, timeout: &str, memory: &str) {
    let vars_path = format!("{}/{}.json", vars_dir, name);
    if !u::file_exists(&vars_path) {
        println!("Scaffolding vars  {}", &vars_path);
        u::mkdir(&vars_dir);
        u::write_str(&vars_path, &default_vars(memory, timeout));
    } else {
        println!("vars for {} exists, skipping", name);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Runtime {
    lang: LangRuntime,
    handler: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Build {
    kind: BuildKind,
    pre: Vec<String>,
    post: Vec<String>,
    command: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Spec {
    name: String,
    runtime: Runtime,
    build: Build,
}

fn make_command(kind: &BuildKind, lang: &Lang) -> String {
    match kind {
        BuildKind::Image => String::from(""),
        BuildKind::Layer => String::from(""),
        BuildKind::Inline | BuildKind::Code => match lang {
            Lang::Python => s!("zip -9 -q lambda.zip *.py"),
            Lang::Ruby => s!("zip -9 -q lambda.zip *.rb"),
            _ => s!(""),
        },
        _ => s!(""),
    }
}

fn function_spec(_dir: &str, name: &str, lang: &str, build_kind: &str) -> Spec {
    let langr = LangRuntime::from_str(lang).unwrap();

    let kind = BuildKind::from_str(build_kind).unwrap();
    let bspec = Build {
        kind: kind.clone(),
        pre: vec![],
        post: vec![],
        command: make_command(&kind, &langr.to_lang()),
    };

    let rspec = Runtime {
        lang: langr,
        handler: s!("handler.handler"),
    };

    Spec {
        name: s!(name),
        runtime: rspec,
        build: bspec,
    }
}

fn write_file(path: &str, data: &str) {
    if u::file_exists(&path) {
        println!("File already exists!");
        std::process::exit(1)
    } else {
        u::write_str(&path, data);
    }
}

fn write(dir: &str, spec: Spec) {
    let yaml = serde_yaml::to_string(&spec).unwrap();
    let path = format!("{}/function.yml", dir);
    write_file(&path, &yaml);
}

fn create(kind: &str, dir: &str, name: &str, runtime: &str, build_kind: &str) {
    let fdir = if kind != "Standalone" {
        format!("{}/{}", dir, name)
    } else {
        dir.to_string()
    };
    u::mkdir(&fdir);
    let spec = function_spec(&fdir, name, runtime, build_kind);
    write(&fdir, spec);
    let langr = LangRuntime::from_str(runtime).unwrap();
    write_handler(&fdir, &langr);
}

fn create_infra(infra_dir: &str, name: &str, memory: &str, timeout: &str) {
    let role_dir = format!("{}/roles", infra_dir);
    let vars_dir = format!("{}/vars", infra_dir);
    let role = Role::default(Entity::Function);
    u::mkdir(&role_dir);
    u::mkdir(&vars_dir);
    write_role(&role_dir, name, &role);
    write_vars(&vars_dir, name, memory, timeout);
}

pub fn scaffold() {
    let name = Text::new("Function name:").prompt();

    let kinds: Vec<&str> = vec!["Namespaced", "Standalone"];

    let function_kind: Result<&str, InquireError> = Select::new("Function kind", kinds)
        .without_help_message()
        .prompt();

    let kind = function_kind.unwrap();

    let dir = u::pwd();
    let namespace = if u::path_exists(&dir, "topology.yml") {
        composer::topology_name(&dir)
    } else {
        if &kind == &"Namespaced" {
            println!(
                "Warn: No topology.yml file found. Please add topology.yml. Continuing anyway.."
            )
        }
        u::basename(&dir)
    };
    let runtimes: Vec<&str> = vec![
        "python3.12",
        "python3.11",
        "python3.10",
        "ruby3.2",
        "rust",
        "clojure",
        "janet",
        "node22",
    ];

    let runtime: Result<&str, InquireError> = Select::new("Select language", runtimes)
        .without_help_message()
        .prompt();

    let deps: Vec<&str> = vec!["Inline", "Image", "Layer", "None"];

    let build_kind: Result<&str, InquireError> = Select::new("Select dependency mechanism", deps)
        .without_help_message()
        .prompt();

    let name = &name.unwrap();
    println!("Creating {} function dir {}", &kind, name);

    create(&kind, &dir, name, &runtime.unwrap(), &build_kind.unwrap());

    let proceed_infra = Confirm::new("Override default runtime parameters (roles, vars) ?")
        .with_default(false)
        .prompt();

    let infra = match proceed_infra {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => false,
    };

    if infra {
        let default_infra_dir = &format!("{}/infrastructure/tc/{}", &u::roots(), namespace);

        let dirs: Vec<&str> = vec![default_infra_dir, "./infra"];

        let infra_dir: Result<&str, InquireError> = Select::new("Select Infra dir", dirs)
            .without_help_message()
            .prompt();

        let memory = Text::new("Memory").with_default("128").prompt();

        let timeout = Text::new("Timeout").with_default("10").prompt();

        create_infra(
            &infra_dir.unwrap(),
            name,
            &memory.unwrap(),
            &timeout.unwrap(),
        );
    }
}
