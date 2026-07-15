use crate::Topology;
use crate::graph;
use petgraph::dot::{Dot, Config};

pub fn pprint(topology: &Topology) {
    let graph = graph::build_digraph(topology);
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
}
