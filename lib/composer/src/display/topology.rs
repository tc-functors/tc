use super::Topology;
use serde_derive::Serialize;
use std::collections::HashMap;
use tabled::{
    Style,
    Table,
    Tabled,
};

#[derive(Tabled, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TopologyCount {
    pub name: String,
    pub kind: String,
    pub nodes: usize,
    pub functions: usize,
    pub events: usize,
    pub queues: usize,
    pub routes: usize,
    pub mutations: usize,
    pub states: usize,
    pub pages: usize,
}

impl TopologyCount {
    pub fn new(topology: &Topology) -> TopologyCount {
        let Topology {
            namespace,
            kind,
            functions,
            mutations,
            events,
            queues,
            routes,
            flow,
            pages,
            ..
        } = topology;
        let mut f: usize = functions.len();
        let mut m: usize = match mutations.get("default") {
            Some(mx) => mx.resolvers.len(),
            _ => 0,
        };
        let mut e: usize = events.len();
        let mut q: usize = queues.len();
        let mut r: usize = routes.len();
        let mut p: usize = pages.len();

        let mut s: usize = if let Some(_f) = flow { 1 } else { 0 };

        let nodes = &topology.nodes;

        for (_, node) in nodes {
            let Topology {
                functions,
                mutations,
                events,
                queues,
                routes,
                flow,
                ..
            } = node;
            f = f + functions.len();
            m = m + match mutations.get("default") {
                Some(mx) => mx.resolvers.len(),
                _ => 0,
            };
            e = e + events.len();
            q = q + queues.len();
            r = r + routes.len();
            p = p + pages.len();
            s = if let Some(_f) = flow { s + 1 } else { s + 0 };
        }

        TopologyCount {
            name: namespace.to_string(),
            kind: kind.to_str(),
            nodes: nodes.len(),
            functions: f,
            events: e,
            queues: q,
            routes: r,
            mutations: m,
            states: s,
            pages: p,
        }
    }
}

pub fn print_stats(topologies: HashMap<String, Topology>) {
    let mut xs: Vec<TopologyCount> = vec![];
    for (_, t) in topologies {
        let c = TopologyCount::new(&t);
        xs.push(c)
    }
    xs.sort_by(|a, b| b.name.cmp(&a.name));
    xs.reverse();
    let table = Table::new(xs).with(Style::psql()).to_string();
    println!("{}", table);
}

pub fn print_stats_json(topologies: HashMap<String, Topology>) {
    let mut xs: Vec<TopologyCount> = vec![];
    for (_, t) in topologies {
        let c = TopologyCount::new(&t);
        xs.push(c)
    }
    xs.sort_by(|a, b| b.name.cmp(&a.name));
    xs.reverse();
    kit::pp_json(&xs);
}


pub fn get_count(topologies: &HashMap<String, Topology>) -> Vec<TopologyCount> {
    let mut xs: Vec<TopologyCount> = vec![];
    for (_, t) in topologies {
        let c = TopologyCount::new(&t);
        xs.push(c)
    }
    xs.sort_by(|a, b| b.name.cmp(&a.name));
    xs.reverse();
    xs
}

pub fn count_str(topology: &Topology) -> String {
    let Topology {
        functions,
        mutations,
        events,
        queues,
        routes,
        pages,
        flow,
        ..
    } = topology;

    let mut f: usize = functions.len();
    let mut m: usize = match mutations.get("default") {
        Some(mx) => mx.resolvers.len(),
        _ => 0,
    };
    let mut e: usize = events.len();
    let mut q: usize = queues.len();
    let mut r: usize = routes.len();
    let mut p: usize = pages.len();
    let mut s: usize = if let Some(_f) = flow { 1 } else { 0 };

    let nodes = &topology.nodes;

    for (_, node) in nodes {
        let Topology {
            functions,
            mutations,
            events,
            queues,
            routes,
            pages,
            flow,
            ..
        } = node;
        f = f + functions.len();
        m = m + match mutations.get("default") {
            Some(mx) => mx.resolvers.len(),
            _ => 0,
        };
        e = e + events.len();
        q = q + queues.len();
        r = r + routes.len();
        p = p + pages.len();
        let snode = if let Some(_) = flow { 1 } else { 0 };
        s = s + snode;
    }

    let msg = format!(
        "nodes: {}, functions: {}, mutations: {}, events: {}, routes: {}, queues: {}, states: {}, pages: {}",
        nodes.len() + 1,
        f,
        m,
        e,
        r,
        q,
        s,
        p
    );
    msg
}
