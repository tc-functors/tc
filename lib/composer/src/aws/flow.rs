use super::{
    template,
};
use compiler::{
    spec::TopologySpec,
};
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use safe_unwrap::safe_unwrap;
mod asl;

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
    pub role: String,
    pub role_arn: String,
    pub log_config: LogConfig,
}


fn generate_asl(spec: &TopologySpec) -> Option<Value> {
    let auto = match spec.auto {
        Some(p) => p,
        None => false,
    };

    match &spec.functions {
        Some(fns) => {
            if auto {
                Some(asl::generate(fns.clone()))
            } else {
                None
            }
        },
        None => None
    }
}


impl Flow {
    pub fn new(fqn: &str, spec: &TopologySpec) -> Option<Flow> {

        let mode = match &spec.mode {
            Some(m) => m.to_string(),
            None => s!("Express"),
        };


        let lg = "/aws/vendedlogs/tc/{{namespace}}-{{sandbox}}/states";
        let log_config = LogConfig {
            group: s!(lg),
            group_arn: template::log_group_arn(&lg),
        };

        let roles = safe_unwrap!("roles not found", spec.roles.clone());
        let role = roles.get("state");
        let role = safe_unwrap!("Role not defined", role);

        match &spec.states {
            Some(definition) => Some(Flow {
                name: s!(fqn),
                arn: template::sfn_arn(fqn),
                definition: definition.clone(),
                mode: mode,
                role: role.name.clone(),
                role_arn: template::role_arn(&role.name),
                log_config: log_config,
            }),
            None =>  {
                let asl = generate_asl(spec);
                match asl {
                    Some(d) => {
                        let role  = format!("tc-base-sfn-{{{{sandbox}}}}");
                        Some(Flow {
                        name: s!(fqn),
                        arn: template::sfn_arn(fqn),
                        definition: d,
                        mode: mode,
                        role_arn: template::role_arn(&role),
                        role: role,
                        log_config: log_config,
                    })
                    },
                    None => None
                }
            }
        }
    }
}
