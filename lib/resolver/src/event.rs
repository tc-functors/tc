use std::collections::HashMap;

use super::{Context, Topology};
use compiler::{Event, Target, TargetKind, Mutation};
use aws::appsync;
use aws::Env;
use kit as u;
use kit::*;

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
async fn get_graphql_arn_id(env: &Env, name: &str) -> Option<String> {
    let client = appsync::make_client(env).await;
    let api = appsync::find_api(&client, name).await;
    match api {
        Some(ap) => {
            let arn = appsync::get_api_endpoint(&client, &ap.id).await;
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

async fn find_mutation(name: &str, mutations: &HashMap<String, Mutation>) -> String {

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

    let fields = types_map.get(output).expect("Not found").keys();
    let mut s: String = s!("");
    for f in fields {
        s.push_str(&format!(
            r"{f}
"
        ))
    }
    let label = u::pascal_case(&name);
    format!(
        r#"mutation {label}($detail: String) {{
  {name}(detail: $detail) {{
    {s}
    createdAt
    updatedAt
  }}
}}"#
    )

}


async fn resolve_target(context: &Context, topology: &Topology, mut target: Target) -> Target {

    let Context { env, .. } = context;
    let name = topology.fqn.clone();

    let target_name = match target.kind {
        TargetKind::Function => fqn_of(context, topology, &target.name),
        TargetKind::Mutation => find_mutation(&target.name, &topology.mutations).await,
        TargetKind::StepFunction => s!(&env.sfn_arn(&name))
    };

    let target_arn = match target.kind {
        TargetKind::Function => fqn_of(context, topology, &target.name),
        TargetKind::Mutation => {
            let id = get_graphql_arn_id(env, &name).await;
            match id {
                Some(gid) => env.graphql_arn(&gid),
                None => s!(""),
            }
        },
        TargetKind::StepFunction => env.sfn_arn(&target_name)
    };
    target.name = target_name;
    target.arn = target_arn;
    target
}


pub async fn resolve(ctx: &Context, topology: &Topology) -> HashMap<String, Event> {

    let Context { sandbox, .. } = ctx;
    let mut events: HashMap<String, Event> = HashMap::new();

    for (name, mut event) in topology.events.clone() {
        let mut targets: Vec<Target> = vec![];


        for target in &event.targets {
            let t = resolve_target(ctx, topology, target.clone()).await;
            targets.push(t);
         }

        event.targets = targets;

        if sandbox == "stable" || event.sandboxes.contains(&sandbox) {
            events.insert(name.to_string(), event.clone());
        }
    }
    events
}
