use crate::{
    Topology,
};
use compiler::Entity;
use petgraph::{
    graph::{
        DiGraph,
    },
    stable_graph::NodeIndex,
};
use std::collections::HashMap;


use petgraph::dot::{
    Config,
    RankDir,
    Dot,
};

#[derive(Eq, Hash, PartialEq)]
struct Source {
    #[allow(dead_code)]
    entity: Entity,
    name: String,
}

#[derive(Clone)]
struct Target {
    #[allow(dead_code)]
    entity: Entity,
    name: String,
}

struct Node {
    #[allow(dead_code)]
    entity: Entity,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    targets: Vec<Target>,
}


fn name_of(s: &str) -> String {
    if s.contains("{{namespace") && s.contains("{{sandbox") {
        let parts: Vec<&str> = s.split("_").collect();
        parts.clone().into_iter().nth(1).unwrap().to_string()
    } else if s.contains("{{sandbox") {
        let parts: Vec<&str> = s.split("_").collect();
        parts.clone().into_iter().nth(0).unwrap().to_string()
    } else {
        s.to_string()
    }
}

fn find_mappings(topology: &Topology) -> HashMap<Source, Vec<Target>> {
    let mut h: HashMap<Source, Vec<Target>> = HashMap::new();
    for (_, route) in &topology.routes {
        let s = Source {
            entity: Entity::Route,
            name: route.path.clone(),
        };
        let t = Target {
            entity: route.target.entity.clone(),
            name: name_of(&route.target.name),
        };
        h.insert(s, vec![t]);
    }

    for (name, event) in &topology.events {
        let s = Source {
            entity: Entity::Event,
            name: name.to_string(),
        };

        let mut xs: Vec<Target> = vec![];
        for target in &event.targets {
            let t = Target {
                entity: target.entity.clone(),
                name: name_of(&target.name),
            };
            xs.push(t);
        }
        h.insert(s, xs);
    }

    for (name, f) in &topology.functions {
        let s = Source {
            entity: Entity::Function,
            name: name.to_string(),
        };
        let mut xs: Vec<Target> = vec![];
        for target in &f.targets {
            let t = Target {
                entity: target.entity.clone(),
                name: name_of(&target.name),
            };
            xs.push(t);
        }
        h.insert(s, xs);
    }

    let maybe_mutations = topology.mutations.get("default");
    if let Some(mutations) = maybe_mutations {
        for (name, resolver) in &mutations.resolvers {
            let s = Source {
                entity: Entity::Mutation,
                name: name.to_string(),
            };
            let t = Target {
                entity: resolver.entity.clone(),
                name: name_of(&resolver.target_name),
            };
            h.insert(s, vec![t]);
        }
    }

    for (name, queue) in &topology.queues {
        let s = Source {
            entity: Entity::Queue,
            name: name_of(&name),
        };
        let mut xs: Vec<Target> = vec![];
        for target in &queue.targets {
            let t = Target {
                entity: target.entity.clone(),
                name: name_of(&target.name),
            };
            xs.push(t);
        }
        h.insert(s, xs);
    }
    h
}

fn make_nodes(mappings: &HashMap<Source, Vec<Target>>) -> HashMap<String, Node> {
    let mut h: HashMap<String, Node> = HashMap::new();
    for (source, targets) in mappings {
        let node = Node {
            entity: source.entity.clone(),
            name: source.name.clone(),
            targets: targets.to_vec(),
        };
        h.insert(source.name.clone(), node);
    }
    h
}

fn make_edges(mappings: &HashMap<Source, Vec<Target>>) -> HashMap<String, String> {
    let mut h: HashMap<String, String> = HashMap::new();
    for (source, targets) in mappings {
        for target in targets {
            h.insert(source.name.clone(), target.name.clone());
        }
    }
    h
}

fn build_digraph(topology: &Topology) -> DiGraph<String, &str> {
    let mut graph = DiGraph::new();

    let mappings = find_mappings(topology);
    let nodes = make_nodes(&mappings);
    let edges = make_edges(&mappings);

    let mut h: HashMap<String, NodeIndex> = HashMap::new();

    for (source, _node) in nodes {
        let n = graph.add_node(source.clone());
        h.insert(source, n);
    }
    for (source, target) in edges {
        if let Some(s) = h.get(&source) {
            if let Some(t) = h.get(&target) {
                graph.add_edge(*s, *t, "");
            }
        }
    }
    graph
}


pub fn pprint(topology: &Topology) {
    let graph = build_digraph(topology);
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel, Config::RankDir(RankDir::LR)]));
}
