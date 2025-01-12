use super::Context;
use kit as u;
use kit::*;
use serde_derive::{Deserialize, Serialize};

fn lambda_arn(acc: &str, region: &str, name: &str) -> String {
    format!("arn:aws:lambda:{}:{}:function:{}", region, acc, name)
}

fn log_group_arn(acc: &str, region: &str, log_group: &str) -> String {
    format!("arn:aws:logs:{}:{}:log-group:{}:*", region, acc, log_group)
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
pub struct Logs {
    pub filter: Filter,
    pub aggregator: Aggregator,
}

impl Logs {
    pub fn new(context: &Context) -> Logs {
        let Context {
            env,
            namespace,
            sandbox,
            ..
        } = context;
        let filter_name = format!("{}_logf_{}", namespace, sandbox);
        let aggregator_states = format!("/aws/vendedlogs/tc/{}-{}/states", namespace, sandbox);
        let aggregator_lambda = format!("/aws/vendedlogs/tc/{}-{}/lambda", namespace, sandbox);
        Logs {
            filter: Filter {
                name: filter_name.clone(),
                arn: lambda_arn(&env.account(), &env.region(), &filter_name),
                id: s!(sandbox),
                expression: u::empty(),
            },
            aggregator: Aggregator {
                states: aggregator_states.clone(),
                lambda: aggregator_lambda.clone(),
                arn: log_group_arn(&env.account(), &env.region(), &aggregator_states),
            },
        }
    }
}
