use compiler::Entity;
use composer::Topology;

use std::collections::HashMap;
use kit::*;

fn attr_of(entity: &Entity) -> String {
    match entity {
        Entity::Function => s!("shape=box, fillcolor=\"#e0e7ff\", style=filled"),
        Entity::Event => s!("shape=ellipse, fillcolor=\"#ecfccb\", style=filled"),
        Entity::Route => s!("shape=box, fillcolor=\"#bbf7d0\" style=filled"),
        Entity::Channel => s!("shape=box, fillcolor=\"#e4e4e7\", style=filled"),
        Entity::Mutation => s!("shape=box, fillcolor=\"#fed7aa\", style=filled"),
        Entity::Queue => s!("shape=box, fillcolor=powderblue, style=filled"),
        Entity::State => s!("shape=box, fillcolor=\"#ffe4e6\", style=filled"),
        Entity::Page => s!("shape=box, fillcolor=mintcream, style=filled"),
        Entity::Trigger => s!("shape=box, fillcolor=mintcream, style=filled"),
        Entity::Schedule => s!("shape=box, fillcolor=mintcream, style=filled"),
    }
}

#[derive(Eq, Hash, PartialEq)]
struct Source {
    entity: Entity,
    name: String
}

struct Target {
    entity: Entity,
    name: String
}

fn name_of(s: &str) -> String {
    if s.contains("{{namespace") && s.contains("{{sandbox") {
        let parts: Vec<&str>= s.split("_").collect();
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
            name: route.path.clone()
        };
        let t = Target {
            entity: route.target.entity.clone(),
            name: name_of(&route.target.name)
        };
        h.insert(s, vec![t]);
    }

    for (name, event) in &topology.events {
        let s = Source {
            entity: Entity::Event,
            name: name.to_string()
        };

        let mut xs: Vec<Target> = vec![];
        for target in &event.targets {
            let t = Target {
                entity: target.entity.clone(),
                name: name_of(&target.name)
            };
            xs.push(t);
        }
        h.insert(s, xs);
    }

    for (name, f) in &topology.functions {

        let s = Source {
            entity: Entity::Function,
            name: name.to_string()
        };
        let mut xs: Vec<Target> = vec![];
        for target in &f.targets {
            let t = Target {
                entity: target.entity.clone(),
                name: name_of(&target.name)
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
                name: name_of(&resolver.target_name)
            };
            h.insert(s, vec![t]);
        }
    }

    for (name, queue) in &topology.queues {
        let s = Source {
            entity: Entity::Queue,
            name: name_of(&name)
        };
        let mut xs: Vec<Target> = vec![];
        for target in &queue.targets {
            let t = Target {
                entity: target.entity.clone(),
                name: name_of(&target.name)
            };
            xs.push(t);
        }
        h.insert(s, xs);
    }
    h
}

fn make_nodes(mappings: &HashMap<Source, Vec<Target>>) -> String {
    let mut s: String = String::from("");
    for (source, targets) in mappings {
        let attr = attr_of(&source.entity);
        let m = format!(r#""{}"[{}]
"#, &source.name, attr);
        s.push_str(&m);
        for target in targets {
            let attr = attr_of(&target.entity);
            let t = format!(r#""{}"[{}]
"#, &target.name, attr);
        s.push_str(&t);
        }
    }
    s
}

fn make_edges(mappings: &HashMap<Source, Vec<Target>>) -> String {
    let mut s: String = String::from("");
    for (source, targets) in mappings {
        for target in targets {
            let m = format!(r#""{}" -> "{}"
"#, &source.name, &target.name);
            s.push_str(&m);
        }
    }
    s
}

pub fn build(topology: &Topology) -> String {
    let mappings = find_mappings(topology);
    let nodes = make_nodes(&mappings);
    let edges = make_edges(&mappings);


    // let orientation = if mappings.len() > 4 {
    //     "LR"
    // } else {
    //     "TB"
    // };
    let orientation = "LR";
    if !nodes.is_empty() && !edges.is_empty() {
        let s = format!(r#"digraph {{
rankdir="{orientation}"
{nodes} {edges}  }}"#);
        s
    } else {
        String::from("")
    }
}
