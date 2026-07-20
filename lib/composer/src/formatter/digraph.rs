use crate::{
    Topology,
    graph,
};

use petgraph::dot::{
    Config,
    RankDir,
    Dot,
};

pub fn pprint(topology: &Topology) {
    let graph = graph::build_digraph(topology);
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel, Config::RankDir(RankDir::LR)]));
}
