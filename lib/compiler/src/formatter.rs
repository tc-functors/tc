use std::collections::HashMap;
use super::Topology;
use tabled::{Tabled, Table, Style};

#[derive(Tabled, Clone, Debug)]
pub struct TopologyCount {
    pub name: String,
    pub nodes: usize,
    pub functions: usize,
    pub events: usize,
    pub queues: usize,
    pub routes: usize,
    pub mutations: usize,
    pub states: usize
}

impl TopologyCount {

    pub fn new(topology: &Topology) -> TopologyCount {

       let Topology { namespace, functions, mutations, events, queues, routes, .. } = topology;
        let mut f: usize = functions.len();
        let mut m: usize = match mutations.get("default") {
            Some(mx) => mx.resolvers.len(),
            _ => 0
        };
        let mut e: usize = events.len();
        let mut q: usize = queues.len();
        let mut r: usize = routes.len();

        let nodes = &topology.nodes;

        for node in nodes {
            let Topology { functions, mutations, events, queues, routes, .. } = node;
            f = f  + functions.len();
            m = m + match mutations.get("default") {
                Some(mx) => mx.resolvers.len(),
                _ => 0
            };
            e = e + events.len();
            q = q + queues.len();
            r = r + routes.len();
        }

        TopologyCount {
            name: namespace.to_string(),
            nodes: nodes.len(),
            functions: f,
            events: e,
            queues: q,
            routes: r,
            mutations: m,
            states: 0
        }
    }
}

pub fn print_topologies(topologies: HashMap<String, Topology>) {
    let mut xs: Vec<TopologyCount> = vec![];
    for (_, t) in topologies {
        let c = TopologyCount::new(&t);
        xs.push(c)
    }
    let table = Table::new(xs).with(Style::psql()).to_string();
    println!("{}", table);
}
