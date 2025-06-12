use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};

fn lambda_arn(name: &str) -> String {
    format!(
        "arn:aws:lambda:{{{{region}}}}:{{{{account}}}}:function:{}",
        name
    )
}

fn log_group_arn(log_group: &str) -> String {
    format!(
        "arn:aws:logs:{{{{region}}}}:{{{{account}}}}:log-group:{}:*",
        log_group
    )
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Filter {
    pub name: String,
    pub arn: String,
    pub id: String,
    pub expression: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Aggregator {
    pub states: String,
    pub lambda: String,
    pub arn: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogConfig {
    pub filter: Filter,
    pub aggregator: Aggregator,
}

impl LogConfig {
    pub fn new() -> LogConfig {
        let filter_name = "{{namespace}}_logf_{{sandbox}}";
        let aggregator_states = "/aws/vendedlogs/tc/{{namespace}}-{{sandbox}}/states";
        let aggregator_lambda = "/aws/vendedlogs/tc/{{namespace}}-{{sandbox}}/lambda";
        LogConfig {
            filter: Filter {
                name: s!(filter_name),
                arn: lambda_arn(filter_name),
                id: s!("{{sandbox}}"),
                expression: u::empty(),
            },
            aggregator: Aggregator {
                states: s!(aggregator_states),
                lambda: s!(aggregator_lambda),
                arn: log_group_arn(aggregator_states),
            },
        }
    }
}
