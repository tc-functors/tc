use crate::{
    Entity,
    Topology,
};

use crate::graph;
use layout::backends::svg::SVGWriter;
use layout::core::utils::save_to_file;
use layout::gv;
use gv::parser::DotParser;
use gv::GraphBuilder;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

pub enum Format {
    Tree,
    Table,
    JSON,
    YAML,
    Dot,
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
            "dot" | "graphviz" => Ok(Format::Dot),
            _ => Ok(Format::JSON),
        }
    }
}

pub fn print_entity(topology: &Topology, entity: Entity, fmt: Format) {
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

pub fn print_graphql(types: &HashMap<String, String>) {
    for (_, v) in types {
        println!("{}", v)
    }
}

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

pub fn print_seqd() {


}
