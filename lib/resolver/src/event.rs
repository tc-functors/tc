use super::{
    Context,
    Topology,
};
use crate::aws;
use authorizer::Auth;
use composer::{
    Entity,
    Event,
    Mutation,
    Target,
};
use kit as u;
use kit::*;
use std::collections::HashMap;

fn fqn_of(context: &Context, topology: &Topology, fn_name: &str) -> String {
    let Topology { functions, .. } = topology;
    for (_, f) in functions {
        if &f.name == fn_name {
            return context.render(&f.fqn);
        }
    }
    return context.render(fn_name);
}

// appsync targets
async fn get_graphql_arn_id(auth: &Auth, name: &str) -> Option<String> {
    let client = aws::appsync::make_client(auth).await;
    let api = aws::appsync::find_api(&client, name).await;
    match api {
        Some(id) => {
            let arn = aws::appsync::get_api_endpoint(&client, &id).await;
            match arn {
                Some(a) => {
                    let tmp = u::split_last(&a, "://");
                    Some(u::split_first(&tmp, "."))
                }
                None => None,
            }
        }
        None => None,
    }
}


fn make_mutation(name: &str, mutations: &HashMap<String, Mutation>) -> String {
    // FIXME: mutations hashmap key is api-name. Using default
    let mutation = mutations.get("default").unwrap();
    let Mutation {
        types_map,
        resolvers,
        ..
    } = mutation;
    let resolver = resolvers.get(name);
    let output = match resolver {
        Some(r) => &r.output,
        None => panic!("resolver output type not defined"),
    };

    let input = match resolver {
        Some(r) => &r.input,
        None => panic!("resolver input type not defined"),
    };

    let fields = types_map.get(output).expect("Not found").keys();
    let mut s: String = s!("");
    for f in fields {
        s.push_str(&format!(
            r"{f}
"
        ))
    }

    let input_fields = types_map.get(input).expect("Not found");
    let detail_type = input_fields.get("detail").expect("Not found");
    let label = u::pascal_case(&name);
    format!(
        r#"mutation {label}($detail: {detail_type}) {{
  {name}(detail: $detail) {{
    {s}
    createdAt
    updatedAt
  }}
}}"#
    )
}

async fn resolve_target(context: &Context, topology: &Topology, mut target: Target) -> Target {
    let Context { auth, .. } = context;
    let name = topology.fqn.clone();

    let target_name = match target.entity {
        Entity::Function => fqn_of(context, topology, &target.name),
        Entity::Mutation => make_mutation(&target.name, &topology.mutations),
        Entity::State => name.clone(),
        Entity::Channel => name.clone(),
        _ => name.clone(),
    };

    let target_arn = match target.entity {
        Entity::Function => auth.lambda_arn(&target_name),
        Entity::Mutation => {
            let id = get_graphql_arn_id(auth, &name).await;
            match id {
                Some(gid) => auth.graphql_arn(&gid),
                None => String::from("none"),
            }
        }
        Entity::State => auth.sfn_arn(&target_name),
        Entity::Channel => target.arn,
        _ => target.arn,
    };
    target.name = target_name;
    target.arn = target_arn;
    target
}

pub async fn resolve(ctx: &Context, topology: &Topology) -> HashMap<String, Event> {
    let Context {
        sandbox, config, ..
    } = ctx;
    let mut events: HashMap<String, Event> = HashMap::new();

    for (name, mut event) in topology.events.clone() {
        let mut targets: Vec<Target> = vec![];

        for target in &event.targets {
            let t = resolve_target(ctx, topology, target.clone()).await;
            targets.push(t);
        }

        event.targets = targets;

        let guard = match std::env::var("TC_FORCE_DEPLOY") {
            Ok(_) => false,
            Err(_) => config.deployer.guard_stable_updates
        };

        if guard {
            if sandbox == "stable" || event.sandboxes.contains(&sandbox) {
                events.insert(name.to_string(), event.clone());
            }
        } else {
            events.insert(name.to_string(), event.clone());
        }
    }
    events
}
