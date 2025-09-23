use super::{
    Role,
    RoleKind,
    role,
    spec::TopologySpec,
    template,
};
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flow {
    pub name: String,
    pub arn: String,
    pub definition: Value,
    pub mode: String,
    pub role: Role,
}

fn read_definition(dir: &str, def: Value) -> Value {
    match def.as_str() {
        Some(p) => {
            let path = format!("{}/{}", dir, &p);
            if (path.ends_with(".json") || path.ends_with(".yml")) && u::file_exists(&path) {
                let data = u::slurp(&path);
                u::json_value(&data)
            } else {
                def
            }
        }
        None => def,
    }
}

fn make_role(infra_dir: &str, fqn: &str) -> Role {
    let role_file = format!("{}/roles/sfn.json", infra_dir);
    let role_name = format!("tc-{}-sfn-role", fqn);
    let policy_name = format!("tc-{}-sfn-policy", fqn);
    if u::file_exists(&role_file) {
        Role::new(RoleKind::StepFunction, &role_file, &role_name, &policy_name)
    } else {
        role::default(RoleKind::StepFunction)
    }
}

impl Flow {
    pub fn new(dir: &str, infra_dir: &str, fqn: &str, spec: &TopologySpec) -> Option<Flow> {
        let def = match &spec.flow {
            Some(f) => Some(read_definition(dir, f.clone())),
            None => spec.states.to_owned(),
        };

        let mode = match &spec.mode {
            Some(m) => m.to_string(),
            None => s!("Express"),
        };

        match def {
            Some(definition) => Some(Flow {
                name: s!(fqn),
                arn: template::sfn_arn(fqn),
                definition: definition,
                mode: mode,
                role: make_role(infra_dir, fqn),
            }),
            None => None,
        }
    }
}
