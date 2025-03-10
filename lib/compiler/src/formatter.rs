use std::collections::HashMap;
use super::Topology;
use tabled::{Tabled, Table, Style};

#[derive(Tabled, Clone, Debug)]
pub struct TopologyCount {
    pub name: String,
    pub functions: usize,
    pub events: usize,
    pub queues: usize,
    pub routes: usize,
    pub mutations: usize
}

impl TopologyCount {

    pub fn new(topology: &Topology) -> TopologyCount {

        let Topology { namespace, functions, mutations, events, queues, routes, .. } = topology;

        let f: usize = functions.len();
        let m: usize = match mutations.get("default") {
            Some(mx) => mx.resolvers.len(),
            _ => 0
        };
        let e: usize = events.len();
        let q: usize = queues.len();
        let r: usize = routes.len();

        TopologyCount {
            name: namespace.to_string(),
            functions: f,
            events: e,
            queues: q,
            routes: r,
            mutations: m
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
