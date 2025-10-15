use crate::{
    Entity,
    Topology,
};
use kit as u;
use kit::*;
use std::{
    collections::HashMap,
    str::FromStr,
};
use ptree::TreeBuilder;
use serde_derive::Serialize;
mod event;
mod function;
mod state;
mod diagram;
pub mod topology;
use crate::graph;
use layout::backends::svg::SVGWriter;
use layout::core::utils::save_to_file;
use layout::gv;
use gv::parser::DotParser;
use gv::GraphBuilder;
use colored::Colorize;

use tabled::{
    Style,
    Table,
    Tabled,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

pub enum Format {
    Tree,
    Table,
    JSON,
    YAML,
    Graphql,
    Dot,
    Graph,
}

impl FromStr for Format {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Format::JSON),
            "tree" => Ok(Format::Tree),
            "table" => Ok(Format::Table),
            "yaml" => Ok(Format::YAML),
            "dot"=> Ok(Format::Dot),
            "graph" => Ok(Format::Graph),
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

fn as_uri(s: &str) -> String {
    if s.starts_with("/") {
        u::gdir(&s)
    } else {
        s.to_string()
    }
}

pub fn print_tree(topology: &Topology) {
    let Topology { namespace, functions, events, routes, .. } = topology;
    let mut t = TreeBuilder::new(s!(namespace.blue()));

    t.begin_child(s!("functions".cyan()));
    for (name, f) in functions {
        t.begin_child(s!(name.green()));
        t.add_empty_child(f.name.clone());
        t.add_empty_child(format!("fqn: {}", f.fqn.clone()));
        t.add_empty_child(format!("role: {}", f.runtime.role.name.clone()));
        t.add_empty_child(format!("uri: {}", as_uri(&f.runtime.uri)));
        t.add_empty_child(format!("build: {}", f.build.kind.to_str()));
        t.end_child();
    }
    t.end_child();

    t.begin_child(s!("events"));
    for (name, _e) in events {
        t.add_empty_child(name.clone());
    }
    t.end_child();

    t.begin_child(s!("routes"));
    for (_name, r) in routes {
        t.add_empty_child(r.path.clone());
    }
    t.end_child();

    // t.begin_child(s!("mutations"));
    // for (_, f) in &topology.mutations.resolvers {
    //     t.add_empty_child(f.name.clone());
    // }

    let tree = t.build();
    kit::print_tree(tree);
}

#[derive(Tabled, Clone, Debug, Serialize)]
struct Version {
    namespace: String,
    version: String,
}

pub fn print_versions(versions: HashMap<String, String>, format: Format) {
    let mut xs: Vec<Version> = vec![];
    for (namespace, version) in versions {
        let v = Version {
            namespace: s!(namespace),
            version: s!(version),
        };
        xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
        xs.reverse();
        xs.push(v)
    }
    match format {
        Format::Table => {
            let table = Table::new(xs).with(Style::psql()).to_string();
            println!("{}", table);
        }
        Format::JSON => u::pp_json(&xs),
        _ => todo!(),
    }
}

//graph

pub fn print_dot(topology: &Topology) {
    let dir = u::pwd();
    let path = format!("{}/output.svg", &dir);

    let dot_str = graph::build(topology);
    let mut parser = DotParser::new(&dot_str);

    let tree = parser.process();

    match tree {
        Result::Err(err) => {
            parser.print_error();
            println!("Error: {}", err);
        }

        Result::Ok(g) => {
            gv::dump_ast(&g);

            let mut gb = GraphBuilder::new();
            gb.visit_graph(&g);
            let mut vg = gb.get();
            let mut svg = SVGWriter::new();
            vg.do_it(false, false, false, &mut svg);
            let content = svg.finalize();
            let res = save_to_file(&path, &content);
            if let Result::Err(err) = res {
                println!("Could not write the file {}", &path);
                println!("Error {}", err);
                return;
            }
        }
    }
    open::that(path).unwrap();
}

pub fn print_graph(topology: &Topology) {
    diagram::generate(topology);
}
