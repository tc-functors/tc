pub mod context;
pub mod display;
pub mod event;
pub mod flow;
pub mod function;
pub mod logs;
pub mod mutation;
pub mod plan;
pub mod queue;
pub mod role;
pub mod route;
pub mod sandbox;
pub mod schedule;
pub mod vars;

pub use context::Context;
pub use event::Event;
pub use flow::Flow;
pub use function::{Function, ContainerTask};
pub use logs::Logs;
pub use mutation::ResolvedMutations;
pub use plan::Plan;
pub use queue::Queue;
pub use role::Role;
pub use route::Route;
pub use sandbox::Sandbox;
pub use schedule::Schedule;
pub use vars::Vars;

use crate::compiler;
use crate::compiler::{FunctionSpec, RuntimeSpec, Topology};
use aws::Env;
use kit as u;
use std::collections::HashMap;

fn maybe_component(component: Option<String>) -> String {
    match component {
        Some(m) => m,
        _ => "all".to_string(),
    }
}

pub fn should_resolve(component: Option<String>) -> bool {
    let component = maybe_component(component);
    match component.as_str() {
        "logs" => false,
        "events" => false,
        "routes" => false,
        "layers" => true,
        "secrets" => false,
        "roles" => false,
        "vars" => true,
        "tags" => false,
        "functions" => true,
        "all" => true,
        "basic" => false,
        "schedule" => true,
        "mutations" => false,
        "config" => false,
        _ => true,
    }
}

pub async fn resolve(
    env: &Env,
    sandbox: Option<String>,
    topology: &Topology,
    resolve: bool,
) -> Vec<Plan> {
    let sandbox = Sandbox::new(sandbox);

    let nodes = &topology.nodes;
    let mut plans: Vec<Plan> = vec![];
    let root = Plan::new(topology, &env, &sandbox, resolve).await;
    plans.push(root);
    for node in nodes {
        let node_plan = Plan::new(&node, &env, &sandbox, resolve).await;
        plans.push(node_plan);
    }
    plans
}

pub async fn just_nodes(sandbox: &str, topology: &Topology) -> Vec<String> {
    let mut nodes: Vec<String> = vec![];
    let root = if topology.hyphenated_names {
        format!("{}-{}", &topology.name, &sandbox)
    } else {
        format!("{}_{}", &topology.name, &sandbox)
    };
    nodes.push(root);
    for node in &topology.nodes {
        let name = if topology.hyphenated_names {
            format!("{}-{}", &node.name, &sandbox)
        } else {
            format!("{}_{}", &node.name, &sandbox)
        };
        nodes.push(name);
    }
    nodes
}

pub fn render(plans: Vec<Plan>, component: &str) -> String {
    let plan = plans.clone().into_iter().nth(0).unwrap();
    match component {
        "functions" => u::pretty_json(plan.functions),
        "logs" => u::pretty_json(plan.logs),
        "flow" => match plan.flow {
            Some(f) => u::pretty_json(f),
            _ => u::empty(),
        },
        "layers" => display::render_layers(&plans),
        "events" => u::pretty_json(plan.events),
        "schedules" => u::pretty_json(plan.schedules),
        "roles" => u::pretty_json(plan.roles),
        "routes" => u::pretty_json(plan.routes),
        "mutations" => u::pretty_json(plan.mutations),
        "vars" => u::pretty_json(plan.functions),
        "basic" => u::pretty_json(plan.version),
        "all" => u::pretty_json(plans),
        _ => {
            if u::file_exists(&component) {
                let fs = plan.functions;
                let f = fs.get(component).unwrap();
                u::pretty_json(f)
            } else {
                u::pretty_json(plans)
            }
        }
    }
}

pub async fn functions(dir: &str, env: &Env, sandbox: Option<String>) -> Vec<String> {
    let topology = compiler::compile(&dir, true);
    let nodes = &topology.nodes;
    let sbox = Sandbox::new(sandbox);
    let plan = Plan::new(&topology, env, &sbox, false).await;

    let mut fns: Vec<String> = vec![];
    for (_, f) in plan.functions {
        fns.push(f.name)
    }

    for node in nodes {
        let node_plan = Plan::new(&node, env, &sbox, false).await;
        for (_, f) in node_plan.functions {
            fns.push(f.name)
        }
    }
    fns
}

pub fn current_function(sandbox: &str) -> Option<String> {
    let dir = u::pwd();
    let topology = compiler::compile(&dir, false);

    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);

    for (cdir, f) in topology.functions {
        if &cdir == &dir {
            return Some(u::stencil(&f.fqn, table));
        }
    }
    None
}

pub fn as_sandbox(sandbox: Option<String>) -> String {
    sandbox::as_sandbox(sandbox)
}

pub async fn name_of(dir: &str, sandbox: &str, kind: &str) -> Option<String> {
    let topology = compiler::compile(&dir, false);
    match kind {
        "step-function" => {
            let nodes = just_nodes(&sandbox, &topology).await;
            let node = nodes.into_iter().nth(0).unwrap();
            Some(node)
        }
        "lambda" | "function" => current_function(sandbox),
        _ => None,
    }
}
