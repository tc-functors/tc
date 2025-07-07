use crate::{
    Entity,
    Topology,
};
use kit as u;
use std::{
    collections::HashMap,
    str::FromStr,
};

mod event;
mod function;
mod mutation;
mod state;
pub mod topology;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

pub enum Format {
    Tree,
    Table,
    JSON,
    YAML,
    Graphql,
}

impl FromStr for Format {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Format::JSON),
            "tree" => Ok(Format::Tree),
            "table" => Ok(Format::Table),
            "yaml" => Ok(Format::YAML),
            "graphql" | "gql" => Ok(Format::Graphql),
            _ => Ok(Format::JSON),
        }
    }
}

pub fn display_entity(entity: Entity, fmt: Format, topology: &Topology) {
    let Topology {
        events,
        routes,
        flow,
        channels,
        ..
    } = topology;

    match entity {
        Entity::State => {
            if let Some(f) = flow {
                match fmt {
                    Format::JSON => u::pp_json(&f),
                    Format::YAML => println!("{}", serde_yaml::to_string(&f).unwrap()),
                    _ => u::pp_json(&f),
                }
            }
        }

        Entity::Route => u::pp_json(routes),
        Entity::Event => u::pp_json(events),
        Entity::Channel => u::pp_json(channels),

        Entity::Function => match fmt {
            Format::Tree => {
                let tree = function::build_tree(topology);
                kit::print_tree(tree);
            }
            Format::JSON => u::pp_json(&topology.functions),
            Format::Table => u::pp_json(&topology.functions),
            _ => todo!(),
        },

        Entity::Mutation => match fmt {
            Format::Graphql => {
                print_graphql(
                    &topology
                        .mutations
                        .values()
                        .into_iter()
                        .nth(0)
                        .unwrap()
                        .types,
                );
            }
            _ => u::pp_json(&topology.mutations),
        },
        Entity::Page => u::pp_json(&topology.pages),
        _ => (),
    }
}

fn display_component(entity: Entity, component: &str, _fmt: Format, topology: &Topology) {
    match entity {
        Entity::Function => function::display_component(topology, component),
        Entity::State => state::display_component(topology, component),
        Entity::Event => event::display_component(topology, component),
        _ => (),
    }
}

pub fn try_display(topology: &Topology, maybe_entity: &str, fmt: Format) {
    let (entity, component) = Entity::as_entity_component(maybe_entity);
    match component {
        Some(c) => display_component(entity, &c, fmt, topology),
        None => display_entity(entity, fmt, topology),
    }
}

fn print_graphql(types: &HashMap<String, String>) {
    for (_, v) in types {
        println!("{}", v)
    }
}
