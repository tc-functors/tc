pub use crate::aws::{
    channel::Channel,
    event::Event,
    flow::Flow,
    function::{
        Function,
    },
    mutation::{
        Mutation,
        Resolver,
    },
    page::Page,
    pool::Pool,
    queue::Queue,
    role::Role,
    route::Route,
    schedule::Schedule,
};

use safe_unwrap::safe_unwrap;
use kit as u;

use crate::aws::{
    channel,
    event,
    mutation,
    pool,
    template,
};
use compiler::{
    spec::{
        TestSpec,
        TopologyKind,
        TopologySpec,
    },
};
use configurator::Config;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashMap,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Topology {
    pub namespace: String,
    pub env: String,
    pub fqn: String,
    pub kind: TopologyKind,
    pub infra: String,
    pub dir: String,
    pub sandbox: String,
    pub version: String,
    pub nodes: HashMap<String, Topology>,
    pub events: HashMap<String, Event>,
    pub routes: HashMap<String, Route>,
    pub functions: HashMap<String, Function>,
    pub mutations: HashMap<String, Mutation>,
    pub schedules: HashMap<String, Schedule>,
    pub queues: HashMap<String, Queue>,
    pub channels: HashMap<String, Channel>,
    pub pools: HashMap<String, Pool>,
    pub pages: HashMap<String, Page>,
    pub tags: HashMap<String, String>,
    pub flow: Option<Flow>,
    pub config: Config,
    pub roles: HashMap<String, Role>,
    pub tests: HashMap<String, TestSpec>,
}

fn make_functions(spec: &TopologySpec) -> HashMap<String, Function> {
    let mut h: HashMap<String, Function> = HashMap::new();

    let dir = safe_unwrap!("Dir not defined", spec.dir.clone());
    if let Some(fns) = &spec.functions {
        for (name, f) in fns {
            let function = Function::new(&dir, &spec.name, &name, &f);
            h.insert(name.to_string(), function);
        }
    }
    h
}

fn make_roles(spec: &TopologySpec) -> HashMap<String, Role> {
    let mut h: HashMap<String, Role> = HashMap::new();
    if let Some(role_specs) = &spec.roles {
        for (name, rs) in role_specs {
            let role = Role::new(&rs);
            h.insert(name.to_string(), role);
        }
    }
    h
}

fn make_events(
    namespace: &str,
    spec: &TopologySpec,
    fqn: &str,
    config: &Config,
    fns: &HashMap<String, Function>,
    resolvers: &HashMap<String, Resolver>,
) -> HashMap<String, Event> {
    let events = &spec.events;
    let mut h: HashMap<String, Event> = HashMap::new();
    if let Some(evs) = events {
        for (name, espec) in evs {
            let targets = event::make_targets(namespace, &name, &espec, fqn, fns, resolvers);
            let skip = espec.doc_only;
            let ev = Event::new(&name, &espec, targets, config, skip);
            h.insert(name.to_string(), ev);
        }
    }
    h
}

fn make_routes(
    spec: &TopologySpec,
    fqn: &str,
    fns: &HashMap<String, Function>,
) -> HashMap<String, Route> {
    let routes = &spec.routes;
    match routes {
        Some(xs) => {
            let mut h: HashMap<String, Route> = HashMap::new();
            for (name, rspec) in xs {
                let skip = rspec.doc_only;
                let route = Route::new(fqn, &name, spec, rspec, fns, skip);
                h.insert(name.to_string(), route);
            }
            h
        }
        None => HashMap::new(),
    }
}

fn make_queues(spec: &TopologySpec, _config: &Config) -> HashMap<String, Queue> {
    let mut h: HashMap<String, Queue> = HashMap::new();
    if let Some(queues) = &spec.queues {
        for (name, qspec) in queues {
            h.insert(name.to_string(), Queue::new(&name, qspec));
        }
    }
    h
}

fn make_mutations(spec: &TopologySpec, _config: &Config) -> HashMap<String, Mutation> {
    let mutations = mutation::make(&spec.name, spec.mutations.to_owned());
    let mut h: HashMap<String, Mutation> = HashMap::new();
    if let Some(ref m) = mutations {
        h.insert(s!("default"), m.clone());
    }
    h
}

fn make_channels(spec: &TopologySpec, _config: &Config) -> HashMap<String, Channel> {
    match &spec.channels {
        Some(c) => channel::make(&spec.name, c.clone()),
        None => HashMap::new(),
    }
}

fn make_pools(spec: &TopologySpec, config: &Config) -> HashMap<String, Pool> {
    let pools = match &spec.pools {
        Some(p) => p.clone(),
        None => vec![],
    };
    match &spec.triggers {
        Some(c) => pool::make(pools, c.clone(), config),
        None => HashMap::new(),
    }
}

fn make_schedules(spec: &TopologySpec, _config: &Config) -> HashMap<String, Schedule> {
    let scheds = match &spec.schedules {
        Some(c) => c,
        None => &HashMap::new()
    };
    let mut h: HashMap<String, Schedule> = HashMap::new();
    for (name, sched_spec) in scheds {
        let s = Schedule::new(&spec.name, &name, &sched_spec);
        h.insert(name.to_string(), s);
    }
    h
}

fn make_pages(spec: &TopologySpec, config: &Config) -> HashMap<String, Page> {
    let mut h: HashMap<String, Page> = HashMap::new();
    if let Some(pspec) = &spec.pages {
        for (name, ps) in pspec {
            let infra_dir = safe_unwrap!("Infra dir not defined", spec.infra.clone());
            let page = Page::new(&name, &spec.name, &infra_dir, ps, config);
            h.insert(name.to_string(), page);
        }
    }
    h
}

fn make_nodes(spec: &TopologySpec) -> HashMap<String, Topology> {
    let mut h: HashMap<String, Topology> = HashMap::new();
    if let Some(nodes) = &spec.children {
        for (name, node) in nodes {
            h.insert(name.to_string(), make(&node));
        }
    }
    h
}

fn make(spec: &TopologySpec) -> Topology {
    let dir =  safe_unwrap!("dir not defined", spec.dir.clone());

    let namespace = spec.name.to_owned();
    let fqn = match &spec.fqn {
        Some(f) => f,
        None => &template::topology_fqn(&namespace)
    };

    let config = match &spec.config {
        Some(c) => c,
        None => &Config::new()
    };

    let kind = match &spec.kind {
        Some(k) => k,
        None => &TopologyKind::Function
    };

    let infra_dir = match &spec.infra {
        Some(d) => d,
        None => &dir
    };

    let tests = match &spec.tests {
        Some(x) => x,
        None => &HashMap::new()
    };
    let functions = make_functions(&spec);
    let mutations = make_mutations(&spec, &config);
    let routes = make_routes(&spec, &fqn, &functions);

    let resolvers = match &mutations.get("default") {
        Some(m) => m.resolvers.clone(),
        None => HashMap::new(),
    };

    let version = match &spec.version {
        Some(v) => v,
        None => "0.0.0"
    };
    let events = make_events(&namespace, &spec, &fqn, &config, &functions, &resolvers);

    let nodes = make_nodes(&spec);

    let tags = match &spec.tags {
        Some(xs) => xs,
        None => &HashMap::new()
    };

    let flow = Flow::new(&fqn, &spec);

    let roles = make_roles(&spec);

    let config = safe_unwrap!("Config not defined", spec.config.clone());

    Topology {
        namespace: namespace,
        env: template::profile(),
        sandbox: template::sandbox(),
        version: version.to_string(),
        fqn: fqn.to_string(),
        infra: infra_dir.to_string(),
        dir: dir.to_string(),
        kind: kind.clone(),
        nodes: nodes,
        roles: roles,
        events: events,
        routes: routes,
        tests: tests.clone(),
        functions: functions,
        schedules:  make_schedules(&spec, &config),
        queues: make_queues(&spec, &config),
        mutations: mutations,
        channels: make_channels(&spec, &config),
        pools: make_pools(&spec, &config),
        pages: make_pages(&spec, &config),
        tags: tags.clone(),
        config: config,
        flow: flow
    }
}

impl Topology {
    pub fn new(spec: &TopologySpec) -> Topology {
        make(spec)
    }

    pub fn functions(&self) -> HashMap<String, Function> {
        let mut fns: HashMap<String, Function> = self.clone().functions;
        for (_, node) in &self.nodes {
            fns.extend(node.clone().functions);
        }
        fns.clone()
    }

    pub fn current_function(&self, dir: &str) -> Option<Function> {
        let fns: HashMap<String, Function> = self.clone().functions;
        for (_, f) in fns {
            if f.dir == dir {
                return Some(f);
            }
        }
        None
    }

    pub fn to_str(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    pub fn pprint(&self) {
        u::pp_json(self)
    }

}
