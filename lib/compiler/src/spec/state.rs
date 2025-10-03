use super::{
    role::RoleSpec
};
use crate::Entity;
use crate::spec::TopologySpec;
use kit as u;
use serde_json::Value;

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

pub fn make_role(infra_dir: &str, namespace: &str, fqn: &str) -> RoleSpec {
    let maybe_role_file = u::any_path(
        vec![
            format!("{}/roles/sfn.json", infra_dir),
            format!("{}/roles/state.json", infra_dir)
        ]
    );
    let role_file = match maybe_role_file {
        Some(r) => r,
        None => format!("{}/roles/sfn.json", infra_dir)
    };

    if u::file_exists(&role_file) {
        RoleSpec::new(Entity::State, &role_file, namespace, fqn)
    } else {
        RoleSpec::provided_by_entity(Entity::State)
    }
}

pub fn make(dir: &str, spec: &TopologySpec) -> Option<Value> {
    match &spec.flow {
        Some(f) => Some(read_definition(dir, f.clone())),
        None => spec.states.to_owned(),
    }
}
