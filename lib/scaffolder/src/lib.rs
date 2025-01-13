
use kit as u;
use configurator::Config;

fn default_vars() -> String {
    format!(
        r#"{{
  "default": {{
    "timeout": 180,
    "memory_size": 128,
    "environment_variables": {{
      "LOG_LEVEL": "INFO"
    }}
  }},
  "dev": {{
    "default": {{
      "timeout": 180,
      "memory_size": 128,
      "environment_variables": {{
	"LOG_LEVEL": "INFO"
      }}
    }}
  }}
}}"#
    )
}

fn function_spec(name: &str) -> String {
    format!(
        r#"{{
  "name": "{name}",
  "description": "",
  "runtime": {{
    "lang": "python3.10",
    "package_type": "zip",
    "handler": "handler.handler",
    "layers": [],
    "extensions": []
  }},
  "tasks": {{
    "build": "zip lambda.zip handler.py",
    "clean": "rm *.zip"
  }}
}}"#
    )
}

fn write_function_spec(name: &str) {
    let path = format!("{}/function.json", &u::pwd());
    if !u::file_exists(&path) {
        let spec = function_spec(name);
        println!("Scaffolding {}/function.json", name);
        u::write_str(&path, &spec);
    } else {
        println!("{}/function.json exists, skipping", name);
    }
}

fn write_role(name: &str, roles_dir: &str, policy_doc: &str) {
    let role_path = format!("{}/{}.json", roles_dir, name);
    if !u::file_exists(&role_path) {
        println!("Scaffolding role {}", &role_path);
        u::mkdir(roles_dir);
        u::write_str(&role_path, policy_doc);
    } else {
        println!("roles for {} exists, skipping", name);
    }
}

fn write_vars(name: &str, vars_dir: &str) {
    let vars_path = format!("{}/{}.json", vars_dir, name);
    if !u::file_exists(&vars_path) {
        println!("Scaffolding vars  {}", &vars_path);
        u::mkdir(&vars_dir);
        u::write_str(&vars_path, &default_vars());
    } else {
        println!("vars for {} exists, skipping", name);
    }
}

pub async fn create_function(name: &str, infra_dir: &str) {
    let role_dir = format!("{}/roles", infra_dir);
    let vars_dir = format!("{}/vars", infra_dir);
    let env = aws::init(Some(String::from("{{env}}")), None, Config::new(None, "")).await;
    write_role(name, &role_dir, &env.base_lambda_policy());
    write_vars(name, &vars_dir);
    write_function_spec(name);
}
