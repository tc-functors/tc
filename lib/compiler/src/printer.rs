use kit as u;
use kit::*;
use colored::Colorize;
use serde_derive::Serialize;
use std::collections::HashMap;
use tabled::{
    Style,
    Table,
    Tabled,
};

use ptree::{
    builder::TreeBuilder,
};
use std::str::FromStr;

use super::{TopologySpec, Entity};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

#[derive(Debug, PartialEq, Eq)]
pub enum Format {
    Tree,
    Table,
    JSON,
    YAML,
    Bincode,
}

impl FromStr for Format {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Format::JSON),
            "tree" => Ok(Format::Tree),
            "table" => Ok(Format::Table),
            "yaml" => Ok(Format::YAML),
            "bincode" | "bin" => Ok(Format::Bincode),
            _ => Ok(Format::JSON),
        }
    }
}

pub fn print_entity(topology: &TopologySpec, entity: Entity, fmt: Format) {
    let TopologySpec {
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

        Entity::Mutation => u::pp_json(&topology.mutations),
        Entity::Page => u::pp_json(&topology.pages),
        _ => (),
    }
}


#[derive(Tabled, Clone, Debug, Serialize)]
struct Version {
    namespace: String,
    version: String,
}

pub fn print_versions(versions: HashMap<String, String>, format: &str) {
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
        "table" => {
            let table = Table::new(xs).with(Style::psql()).to_string();
            println!("{}", table);
        }
        "json" => u::pp_json(&xs),
        &_ => todo!(),
    }
}

fn as_uri(s: Option<String>) -> String {
    match s {
        Some(p) => {
            if p.starts_with("/") {
                u::gdir(&p)
            } else {
                p
            }
        },
        None => s!("")
    }
}

pub fn print_tree(ts: &TopologySpec) {
    let mut t = TreeBuilder::new(s!(ts.name.blue()));

    if let Some(fns) = &ts.functions {
        t.begin_child(s!("functions".cyan()));
        for (name, f) in fns {
            t.begin_child(s!(name.green()));
            t.add_empty_child(f.name.clone());
            t.add_empty_child(format!("fqn: {}", u::sw(f.fqn.clone())));
            if let Some(runtime) = &f.runtime {
                if let Some(rs) = &runtime.role_spec {
                    t.add_empty_child(format!("role: {}", rs.name.clone()));
                }
                t.add_empty_child(format!("uri: {}",
                                          as_uri(runtime.uri.clone())));

            }
            if let Some(build) = &f.build {
                t.add_empty_child(format!("build: {}", build.kind.to_str()));
            }
            t.end_child();
        }
        t.end_child();
    }

    if let Some(evs) = &ts.events {
        t.begin_child(s!("events"));
        for (name, _e) in evs {
            t.add_empty_child(name.clone());
        }
        t.end_child();
    }

    if let Some(rs) = &ts.routes {
        t.begin_child(s!("routes"));
        for (_name, r) in rs {
            t.add_empty_child(u::sw(r.path.clone()));
        }
        t.end_child();
    }

    // t.begin_child(s!("mutations"));
    // for (_, f) in &topology.mutations.resolvers {
    //     t.add_empty_child(f.name.clone());
    // }

    let tree = t.build();
    kit::print_tree(tree);
}

 #[derive(Tabled, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TopologyCount {
    pub name: String,
    pub kind: String,
    pub nodes: usize,
    pub functions: usize,
    pub events: usize,
    pub queues: usize,
    pub routes: usize,
    pub mutations: usize,
    pub states: usize,
    pub pages: usize,
}

impl TopologyCount {
    pub fn new(topology: &TopologySpec) -> TopologyCount {
        let TopologySpec {
            name,
            kind,
            children,
            functions,
            mutations,
            events,
            queues,
            routes,
            states,
            pages,
            ..
        } = topology;
        let mut f: usize = match functions {
            Some(fx) => fx.len(),
            None => 0
        };
        let mut m: usize = match mutations {
            Some(mx) => mx.resolvers.len(),
            None => 0,
        };
        let mut e: usize = match events {
            Some(ex) => ex.len(),
            None => 0
        };
        let mut q: usize = match queues {
            Some(qx) => qx.len(),
            None => 0
        };
        let mut r: usize = match routes {
            Some(rx) => rx.len(),
            None => 0
        };

        let mut p: usize = match pages {
            Some(px) => px.len(),
            None => 0
        };

        let mut n: usize = match &topology.children {
            Some(m) => m.len(),
            None => 0
        };

        let snode = if let Some(_) = states { 1 } else { 0 };

        let child_nodes = match children {
            Some(c) => c,
            None => &HashMap::new()
        };

        for (_, node) in child_nodes {
            let TopologySpec {
                functions,
                mutations,
                events,
                queues,
                routes,
                pages,
                states,
                ..
            } = node;
            n = n + 1;


            if let Some(fns) = functions {
                f = f + fns.len();
            }

            if let Some(mxs) = mutations {
                m = m + mxs.resolvers.len();
            }

            if let Some(evs) = events {
                e = e + evs.len();
            }

            if let Some(qs) = queues {
                q = q + qs.len();
            }

            if let Some(rs) = routes {
                r = r + rs.len();
            }

            if let Some(ps) = pages {
                p = p + ps.len();
            }

            if let Some(_) = states { snode + 1 } else { snode + 0 };
        }

        let kind = match kind {
            Some(k) => &k.to_str(),
            None => "default",
        };

        TopologyCount {
            name: name.to_string(),
            kind: kind.to_string(),
            nodes: n,
            functions: f,
            events: e,
            queues: q,
            routes: r,
            mutations: m,
            states: snode,
            pages: p
        }
    }
}

pub fn print_count(topologies: HashMap<String, TopologySpec>) {
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
