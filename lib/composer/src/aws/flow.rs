use super::{
    role::Role,
    template,
};
use compiler::{
    Entity,
    spec::TopologySpec,
};
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

mod sfn;
mod sfn_ext;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogConfig {
    pub group: String,
    pub group_arn: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Flow {
    pub name: String,
    pub arn: String,
    pub definition: Value,
    pub mode: String,
    pub role: Role,
    pub log_config: LogConfig,
}

fn make_role(infra_dir: &str, namespace: &str, fqn: &str) -> Role {
    let role_file = format!("{}/roles/sfn.json", infra_dir);
    if u::file_exists(&role_file) {
        Role::new(Entity::State, &role_file, namespace, fqn)
    } else {
        Role::provided_by_entity(Entity::State)
    }
}

fn find_definition(dir: &str, spec: &TopologySpec) -> Option<Value> {
    let auto = match spec.auto {
        Some(p) => p,
        None => false,
    };

    match &spec.flow {
        Some(f) => Some(sfn_ext::read(dir, f.clone())),
        None => match &spec.states {
            Some(s) => Some(s.clone()),
            None => match &spec.functions {
                Some(fns) => {
                    if auto {
                        Some(sfn::read(fns.clone()))
                    } else {
                        None
                    }
                }
                None => None,
            },
        },
    }
}

impl Flow {
    pub fn new(dir: &str, infra_dir: &str, fqn: &str, spec: &TopologySpec) -> Option<Flow> {
        let def = find_definition(dir, spec);

        let mode = match &spec.mode {
            Some(m) => m.to_string(),
            None => s!("Express"),
        };

        let lg = "/aws/vendedlogs/tc/{{namespace}}-{{sandbox}}/states";
        let log_config = LogConfig {
            group: s!(lg),
            group_arn: template::log_group_arn(&lg),
        };

        match def {
            Some(definition) => Some(Flow {
                name: s!(fqn),
                arn: template::sfn_arn(fqn),
                definition: definition,
                mode: mode,
                role: make_role(infra_dir, &spec.name, fqn),
                log_config: log_config,
            }),
            None => None,
        }
    }
}
