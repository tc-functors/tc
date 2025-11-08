use compiler::TopologySpec;
use composer::sequence::Connector;
use composer::{Topology, Event, Route, Function};
use rand::Rng;
use kit::*;

use serde_derive::{
    Deserialize,
    Serialize,
};

use std::collections::HashMap;


pub fn names_of(connectors: &Vec<Connector>) -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    for c in connectors {
        xs.push(c.source.clone());
        xs.push(c.target.clone());
    }
    xs.sort();
    xs.dedup();
    xs
}

pub fn gen_sequence(connectors: &Vec<Connector>) -> String {
    let mut s: String = String::from("");
    let names = names_of(connectors);
    let p = format!(r#"sequenceDiagram
"#);
    s.push_str(&p);

    for name in &names {
        let p = format!(r#"participant {name}
"#);
        s.push_str(&p);
    }

    for c in connectors {
        let p = format!(r#"{}->>{}: {}
"#, c.source, c.target, c.message);
        s.push_str(&p);
    }

    s
}


pub fn gen_flow(connectors: &Vec<Connector>) -> String {
    let mut s: String = String::from("");

    let names = names_of(connectors);

    for c in connectors {
        let f = format!(
            r#"
{}--{}-->{}
"#, &c.source, &c.message, &c.target
        );
        s.push_str(&f);
    }


    for name in &names {
        let clicky = format!(
            r#"click {name} callback
"#);
        s.push_str(&clicky);
    }

    let mut style = format!(
            r#"
    classDef red fill:#ffefdf,color:#000,stroke:#333;
    classDef blue fill:#e4fbfc,color:#000,stroke:#333;
    classDef bing fill:#f1edff,color:#000,stroke:#333;
    classDef chan fill:#deffe5,color:#000,stroke:#333;
    classDef c1 fill:#DE8F5F,color:#000,stroke:#333;
    classDef c2 fill:#FFB26F,color:#000,stroke:#333;
    classDef c3 fill:#F1C27B,color:#000,stroke:#333;
    classDef c4 fill:#FFD966,color:#000,stroke:#333;
"#
        );
    let strings = vec!["red", "blue", "bing", "chan", "c1", "c2", "c3", "c4"];

    for name in &names {
        let random_class = &strings[rand::rng().random_range(0..strings.len())];
        let p = format!(r#"
class {name} {random_class}
"#);
        style.push_str(&p);
    }
    s.push_str(&style);
    let mermaid_str = format!(
        r#"
flowchart LR

{s}
"#
    );
    mermaid_str
}


// tree

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    pub path: String,
    pub group: String,
    pub name: String,
    pub detail: String

}

pub fn build_tree(topologies: &HashMap<String, Topology>) -> Vec<Node> {
    let mut xs: Vec<Node> = vec![];
    let mut pindex: u8 = 1;
    for (name, topology) in topologies {
        let node = Node {
            path: format!("{}", pindex),
            group: s!("node"),
            name: name.to_string(),
            detail: serde_json::to_string_pretty(&topology).unwrap()
        };
        xs.push(node);

        if topology.events.len() > 0 {

            xs.push(Node { path: format!("{}.1", pindex), group: s!("events"),
                           name: s!("events"), detail: s!("{}")});
            let mut index: u8 = 1;
            for (n, e) in &topology.events {
                let node = Node {
                    path: format!("{}.1.{}", pindex, index),
                    group: s!("events"),
                    name: n.to_string(),
                    detail: serde_json::to_string_pretty(&e).unwrap()
                };
                xs.push(node);
                index += 1;
            }
        }

        if topology.routes.len() > 0 {
            let mut index: u8 = 1;
            xs.push(Node { path: format!("{}.2", pindex), group: s!("routes"), name: s!("routes"), detail: s!("")});
            for (_n, r) in &topology.routes {
                let node = Node {
                    path: format!("{}.2.{}", pindex, index),
                    group: s!("routes"),
                    name: format!("{} {}", r.method, r.path),
                    detail: serde_json::to_string_pretty(&r).unwrap()
                };
                xs.push(node);
                index += 1;
            }
        }

        if topology.functions.len() > 0 {
            let mut index: u8 = 1;
            xs.push(Node { path: format!("{}.3", pindex), group: s!("functions"), name: s!("functions"), detail: s!("")});
            for (n, f) in &topology.functions {
                let node = Node {
                    path: format!("{}.3.{}", pindex, index),
                    group: s!("functions"),
                    name: n.to_string(),
                    detail: serde_json::to_string_pretty(&f).unwrap()
                };
                xs.push(node);
                index += 1;
            }
        }


        pindex += 1;
    }
    xs
}

pub fn build_shallow_tree(tspecs: &HashMap<String, TopologySpec>) -> Vec<Node> {
    let mut xs: Vec<Node> = vec![];
    let mut pindex: u8 = 1;
    let events: HashMap<String, Event> = HashMap::new();
    let routes: HashMap<String, Route> = HashMap::new();
    let functions: HashMap<String, Function> = HashMap::new();

    for (name, ts) in tspecs {
        let node = Node {
            path: format!("{}", pindex),
            group: s!(name),
            name: name.to_string(),
            detail: String::from("")
        };
        xs.push(node);

        xs.push(
            Node {
                path: format!("{}.1", pindex),
                group: s!(name),
                name: s!("topology.yml"),
                detail: s!("{}")
            });

        let functions = match &ts.functions {
            Some(f) => f,
            None => &HashMap::new()
        };


        println!("----- {:?}", &functions);

        for (fname, fspec) in functions {
            let mut findex: u8 = 2;

            xs.push(
                Node {
                    path: format!("{}.{}", pindex, findex),
                    group: s!(name),
                    name: s!(fname),
                    detail: s!("{}")
                });

            xs.push(
                Node {
                    path: format!("{}.{}.1", pindex, findex),
                    group: s!(fname),
                    name: s!("handler.py"),
                    detail: format!(r#"
def handler(input, context):
  return {{'status': 'ok'}}
"#)});

            xs.push(
                Node {
                    path: format!("{}.{}.2", pindex, findex),
                    group: s!(fname),
                    name: s!("function.yml"),
                    detail: format!(r#"
name: {fname}
runtime:
  lang: python3.11
  package_type: zip
  handler: handler.handler
build:
  kind: Code
  build: zip -9 -q lambda.zip *.py
"#)
                });
            findex += 1;
        }

        pindex += 1;
    }
    xs
}
