use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Context, Topology};
use crate::compiler::mutation::{Mutations, Resolver};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ResolvedMutations {
    pub api_name: String,
    pub authorizer: String,
    pub types: HashMap<String, String>,
    pub resolvers: HashMap<String, Resolver>,
}

fn resolve_target(context: &Context, kind: &str, target: &str) -> String {
    let target = context.render(&target);
    let Context { config, env, .. } = context;
    match kind {
        "function" => env.lambda_arn(&target),
        "event" => env.event_bus_arn(&config.eventbridge.bus),
        _ => target,
    }
}

pub async fn make(context: &Context, topology: &Topology) -> Option<ResolvedMutations> {
    let mutations = topology.mutations.to_owned();
    match mutations {
        Some(mutations) => {
            let Mutations {
                authorizer,
                types,
                resolvers,
                ..
            } = mutations;
            let r_authorizer = context.render(&authorizer);
            let mut resv: HashMap<String, Resolver> = HashMap::new();
            for (type_name, resolver) in resolvers {
                let Resolver {
                    kind,
                    name,
                    target,
                    input,
                    output,
                    ..
                } = resolver;
                let r = Resolver {
                    kind: kind.to_owned(),
                    name: name,
                    target: resolve_target(context, &kind, &target),
                    input: input,
                    output: output,
                };
                resv.insert(type_name, r);
            }

            let m = ResolvedMutations {
                api_name: format!("{}_{}", &topology.name, &context.sandbox),
                authorizer: r_authorizer,
                types: types,
                resolvers: resv,
            };
            Some(m)
        }
        None => None,
    }
}
