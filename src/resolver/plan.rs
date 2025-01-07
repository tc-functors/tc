use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Context, Sandbox};
use super::{Event, Flow, Function, Logs, Queue, Role, Route, Schedule};
use crate::compiler::Topology;
use crate::resolver::mutation::ResolvedMutations;
use crate::resolver::{event, flow, function, mutation, queue, role, route, schedule};
use aws::Env;
use kit as u;
use kit::*;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Plan {
    pub name: String,
    pub namespace: String,
    pub kind: String,
    pub version: String,
    pub dir: String,
    pub revision: String,
    pub sandbox: Sandbox,
    pub env: Env,
    pub logs: Logs,
    pub flow: Option<Flow>,
    pub events: Vec<Event>,
    pub schedules: Vec<Schedule>,
    pub routes: HashMap<String, Route>,
    #[serde(skip_serializing)]
    pub roles: HashMap<String, Role>,
    pub functions: HashMap<String, Function>,
    pub mutations: Option<ResolvedMutations>,
    pub queues: Vec<Queue>,
}

fn make_tags(context: &Context, version: &str) -> HashMap<String, String> {
    let tc_version = option_env!("PROJECT_VERSION")
        .unwrap_or(env!("CARGO_PKG_VERSION"))
        .to_string();
    let Context {
        namespace, sandbox, ..
    } = context;
    let mut h: HashMap<String, String> = HashMap::new();
    h.insert(s!("namespace"), s!(namespace));
    h.insert(s!("sandbox"), s!(sandbox));
    h.insert(s!("version"), s!(version));
    h.insert(s!("deployer"), s!("tc"));
    h.insert(s!("updated_at"), u::utc_now());
    h.insert(s!("tc_version"), tc_version);
    h
}

impl Plan {
    pub async fn new(topology: &Topology, env: &Env, sandbox: &Sandbox, resolve: bool) -> Plan {
        let rev = git::current_revision(&topology.dir);
        let namespace = &topology.name;
        let revision = rev.clone();
        let current_version = git::current_version(&namespace);

        let name = if topology.hyphenated_names {
            format!("{}-{}", &topology.name, sandbox.name)
        } else {
            format!("{}_{}", &topology.name, sandbox.name)
        };

        let mut context = Context {
            env: env.to_owned(),
            namespace: namespace.to_owned(),
            sandbox: sandbox.name.to_owned(),
            name: name.to_owned(),
            resolve: resolve,
            config: topology.config.to_owned(),
        };

        context.resolve_config();

        let given_mode = topology.mode.to_owned();
        let mode = match given_mode {
            Some(m) => m,
            None => s!("Express"),
        };

        let tags = make_tags(&context, &current_version);
        let flow = flow::make(&context, &topology, &mode, &tags);
        let routes = route::make(&context, &topology);
        let roles = role::make(&context, &topology);
        let events = event::make(&context, &topology).await;
        let schedules = schedule::make(&context, &topology);
        let functions = function::make(&context, &topology, &tags).await;
        let mutations = mutation::make(&context, &topology).await;
        let queues = queue::make(&context, &topology).await;
        let logs = Logs::new(&context);

        Plan {
            kind: topology.kind.to_owned(),
            name: name,
            namespace: topology.name.to_owned(),
            dir: topology.dir.to_owned(),
            version: current_version,
            revision: revision,
            env: env.clone(),
            sandbox: sandbox.clone(),
            functions: functions,
            events: events,
            schedules: schedules,
            flow: flow,
            routes: routes,
            mutations: mutations,
            queues: queues,
            roles: roles,
            logs: logs,
        }
    }
}
