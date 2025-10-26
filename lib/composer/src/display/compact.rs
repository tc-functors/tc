use serde_derive::{
    Deserialize,
    Serialize,
};

use std::collections::HashMap;
use crate::{
    Route,
    Event,
    Page,
    Function,
    Queue,
    Channel,
    Topology,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QueueItem {
    pub name: String,
    pub targets: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelItem {
    pub namespace: String,
    pub name: String,
    pub targets: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PageItem  {
    pub namespace: String,
    pub name: String,
    pub bucket: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StateItem {
    pub namespace: String,
    pub mode: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MutationItem {
    pub namespace: String,
    pub name: String,
    pub kind: String,
    pub target: String,
    pub input: String,
    pub output: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionItem {
    pub namespace: String,
    pub name: String,
    pub package_type: String,
    pub dir: String,
    pub fqn: String,
    pub layers: Vec<String>,
    pub memory: i32,
    pub timeout: i32,
    pub runtime: String,
    pub role: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteItem {
    pub namespace: String,
    pub method: String,
    pub path: String,
    pub gateway: String,
    pub authorizer: String,
    pub target_kind: String,
    pub target_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventItem {
    pub namespace: String,
    pub name: String,
    pub rule_name: String,
    pub pattern: String,
    pub targets: HashMap<String, String>,
}

fn build_queues(_namespace: &str, queues: &HashMap<String, Queue>) -> Vec<QueueItem> {
    let mut xs: Vec<QueueItem> = vec![];
    for (_, queue) in queues {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &queue.targets {
            targets.insert(t.entity.to_str(), t.name.clone());
        }
        let e = QueueItem {
            name: queue.name.to_string(),
            targets: targets
        };
        xs.push(e);
    }
    xs
}


fn build_pages(namespace: &str, rs: &HashMap<String, Page>) -> Vec<PageItem> {
    let mut xs: Vec<PageItem> = vec![];

    for (name, page) in rs {
        let e = PageItem {
            namespace: namespace.to_string(),
            name: name.to_string(),
            bucket: page.bucket.clone(),
        };
        xs.push(e);
    }
    xs
}


fn build_channels(namespace: &str, channels: &HashMap<String, Channel>) -> Vec<ChannelItem> {
    let mut xs: Vec<ChannelItem> = vec![];
    for (_, channel) in channels {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &channel.targets {
            targets.insert(t.entity.to_str(), t.name.clone());
        }
        let e = ChannelItem {
            namespace: namespace.to_string(),
            name: channel.name.to_string(),
            targets: targets,
        };
        xs.push(e);
    }
    xs
}


fn build_states(topology: &Topology) -> Vec<StateItem> {
    let mut xs: Vec<StateItem> = vec![];

    if let Some(flow) = &topology.flow {
        let item = StateItem {
            namespace: flow.name.clone(),
            mode: flow.mode.clone(),
            role: flow.role.name.clone(),
        };
        xs.push(item);
    }
    xs
}


fn build_mutations(topology: &Topology) -> Vec<MutationItem> {
    let mut xs: Vec<MutationItem> = vec![];

    for (_, mutation) in &topology.mutations {
        for (_, resolver) in &mutation.resolvers {
            let e = MutationItem {
                namespace: topology.namespace.clone(),
                name: resolver.name.clone(),
                kind: resolver.entity.to_str(),
                target: resolver.target_name.clone(),
                input: resolver.input.clone(),
                output: resolver.output.clone(),
            };
            xs.push(e);
        }
    }
    xs
}

fn build_functions(namespace: &str, fns: &HashMap<String, Function>) -> Vec<FunctionItem> {
    let mut xs: Vec<FunctionItem> = vec![];
    for (dir, f) in fns {
        let fun = FunctionItem {
            namespace: namespace.to_string(),
            name: f.actual_name.clone(),
            dir: dir.to_string(),
            fqn: f.fqn.clone(),
            package_type: f.runtime.package_type.clone(),
            layers: f.runtime.layers.clone(),
            memory: f.runtime.memory_size.unwrap(),
            timeout: f.runtime.timeout.unwrap(),
            runtime: f.runtime.lang.to_str(),
            role: f.runtime.role.name.clone(),
        };
        xs.push(fun);
    }
    xs
}


fn build_events(namespace: &str, evs: &HashMap<String, Event>) -> Vec<EventItem> {
    let mut xs: Vec<EventItem> = vec![];
    for (_, event) in evs {
        let mut targets: HashMap<String, String> = HashMap::new();
        for t in &event.targets {
            targets.insert(t.entity.to_str(), t.name.clone());
        }
        let e = EventItem {
            namespace: namespace.to_string(),
            name: event.name.clone(),
            rule_name: event.rule_name.clone(),
            pattern: serde_json::to_string(&event.pattern).unwrap(),
            targets: targets,
        };
        xs.push(e);
    }
    xs
}

fn build_routes(namespace: &str, rs: &HashMap<String, Route>) -> Vec<RouteItem> {
    let mut xs: Vec<RouteItem> = vec![];

    for (_, route) in rs {
        let e = RouteItem {
            namespace: namespace.to_string(),
            method: route.method.clone(),
            path: route.path.clone(),
            gateway: route.gateway.clone(),
            authorizer: match &route.authorizer {
                Some(auth) => auth.name.clone(),
                None => String::from(""),
            },
            target_kind: route.target.entity.to_str(),
            target_name: route.target.name.clone(),
        };
        xs.push(e);
    }
    xs
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompactTopology {
    pub namespace: String,
    pub events: Vec<EventItem>,
    pub routes: Vec<RouteItem>,
    pub queues: Vec<QueueItem>,
    pub channels: Vec<ChannelItem>,
    pub pages: Vec<PageItem>,
    pub states: Vec<StateItem>,
    pub functions: Vec<FunctionItem>,
    pub mutations: Vec<MutationItem>
}

pub fn build(topologies: &HashMap<String, Topology>) -> Vec<CompactTopology> {
    let mut tops: Vec<CompactTopology> = vec![];
    for (_, topology) in topologies {

        let mut routes = build_routes(&topology.namespace, &topology.routes);
        let mut events = build_events(&topology.namespace, &topology.events);
        let mut functions = build_functions(&topology.namespace, &topology.functions);
        let mut mutations = build_mutations(&topology);
        let mut states = build_states(&topology);
        let mut channels = build_channels(&topology.namespace, &topology.channels);
        let mut pages = build_pages(&topology.namespace, &topology.pages);
        let mut queues = build_queues(&topology.namespace, &topology.queues);

        for (_, node) in &topology.nodes {

            let rs = build_routes(&node.namespace, &node.routes);
            routes.extend(rs);

            let es = build_events(&node.namespace, &node.events);
            events.extend(es);

            let fs = build_functions(&node.namespace, &node.functions);
            functions.extend(fs);

            let ms = build_mutations(&node);
            mutations.extend(ms);

            let ss = build_states(&node);
            states.extend(ss);

            let cs = build_channels(&node.namespace, &node.channels);
            channels.extend(cs);

            let ps = build_pages(&node.namespace, &node.pages);
            pages.extend(ps);

            let qs = build_queues(&node.namespace, &node.queues);
            queues.extend(qs);
        }


        let ct = CompactTopology {
            namespace: topology.namespace.clone(),
            routes: routes,
            events: events,
            functions: functions,
            mutations: mutations,
            states: states,
            channels: channels,
            pages: pages,
            queues: queues
        };
        tops.push(ct);
    }
    tops
}
