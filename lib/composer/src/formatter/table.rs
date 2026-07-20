use crate::Topology;
use crate::counter::TopologyCount;
use serde_derive::Serialize;
use tabled::{
    Style,
    Table,
    Tabled,
};
use std::collections::HashMap;

#[derive(Tabled, Clone, Debug, Serialize)]
struct EntityTarget {
    entity: String,
    name: String,
    target_entity: String,
    target_name: String,
}

pub fn pprint(topology: &Topology) {
    let mut xs: Vec<EntityTarget> = vec![];
    for (name, f) in &topology.functions {
        if f.targets.is_empty() {
            let t = EntityTarget {
                entity: String::from("function"),
                name: name.to_string(),
                target_entity: String::from(""),
                target_name: String::from(""),
            };
            xs.push(t)
        } else {
            for target in &f.targets {
                let t = EntityTarget {
                    entity: String::from("function"),
                    name: name.to_string(),
                    target_entity: target.entity.to_str(),
                    target_name: target.name.clone(),
                };
                xs.push(t)
            }
        }
    }
    for (name, e) in &topology.events {
        for target in &e.targets {
            let t = EntityTarget {
                entity: String::from("event"),
                name: name.to_string(),
                target_entity: target.entity.to_str(),
                target_name: target.name.clone(),
            };
            xs.push(t)
        }
    }

    for (name, r) in &topology.routes {
        let t = EntityTarget {
            entity: String::from("route"),
            name: name.to_string(),
            target_entity: r.target.entity.to_str(),
            target_name: r.target.name.clone(),
        };
        xs.push(t)
    }

    let table = Table::new(xs).with(Style::psql()).to_string();
    println!("{}", table);
}


pub fn pprint_recursive(topologies: &HashMap<String, Topology>) {
    let mut xs: Vec<TopologyCount> = vec![];
    for (_, topology) in topologies {
        let tc = TopologyCount::new(&topology);
        xs.push(tc)
    }
    let table = Table::new(xs).with(Style::psql()).to_string();
    println!("{}", table);
}
