use crate::{
    Entity,
    Topology,
};
mod ascii;
pub mod compact;
mod digraph;
mod icepanel;
mod table;
mod tree;

use crate::TopologyCount;
use kit as u;
use std::{
    collections::HashMap,
    str::FromStr,
    string::ParseError,
};
use tabled::{
    Style,
    Table,
};

pub enum Format {
    Tree,
    Table,
    JSON,
    Dot,
    Icepanel,
    Compact,
    Ascii,
}

impl FromStr for Format {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Format::JSON),
            "tree" => Ok(Format::Tree),
            "table" => Ok(Format::Table),
            "dot" | "digraph" => Ok(Format::Dot),
            "ascii" => Ok(Format::Ascii),
            "icepanel" => Ok(Format::Icepanel),
            "compact" => Ok(Format::Compact),
            _ => Ok(Format::JSON),
        }
    }
}

pub fn pprint(topology: &Topology, fmt: &str) {
    let format = Format::from_str(fmt).unwrap();
    match format {
        Format::JSON => u::pp_json(topology),
        Format::Tree => tree::pprint(topology),
        Format::Table => table::pprint(topology),
        Format::Dot => digraph::pprint(topology),
        Format::Icepanel => icepanel::pprint(topology),
        Format::Compact => compact::pprint(topology),
        Format::Ascii => ascii::pprint(topology),
    }
}

pub fn pprint_recursive(topologies: &HashMap<String, Topology>, fmt: &str) {
    let format = Format::from_str(fmt).unwrap();
    match format {
        Format::JSON => u::pp_json(topologies),
        Format::Tree => tree::pprint_recursive(topologies),
        Format::Table => todo!(),
        Format::Dot => todo!(),
        Format::Icepanel => icepanel::pprint_recursive(topologies),
        Format::Compact => compact::pprint_recursive(topologies),
        Format::Ascii => todo!(),
    }
}

pub fn pprint_entity(topology: &Topology, entity: Entity) {
    match entity {
        Entity::Function => u::pp_json(topology.functions.clone()),
        Entity::Event => u::pp_json(topology.events.clone()),
        Entity::Route => u::pp_json(topology.routes.clone()),
        Entity::Queue => u::pp_json(topology.queues.clone()),
        Entity::Channel => u::pp_json(topology.channels.clone()),
        Entity::Page => u::pp_json(topology.pages.clone()),
        Entity::State => {
            if let Some(f) = &topology.flow {
                let out = serde_yaml::to_string(&f).unwrap();
                println!("{}", &out);
            }
        }
        Entity::Mutation => {
            let types = topology
                .mutations
                .values()
                .into_iter()
                .nth(0)
                .unwrap()
                .types
                .clone();
            for (_, v) in types {
                println!("{}", v)
            }
        }
        _ => todo!(),
    }
}

pub fn pprint_component(topology: &Topology, component: &str) {
    match component {
        "transducer" => u::pp_json(&topology.transducer),
        "roles" => u::pp_json(&topology.roles),
        "base" => u::pp_json(&topology.base_roles),
        _ => todo!(),
    }
}

pub fn pprint_stats(topologies: &HashMap<String, Topology>) {
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
