use composer::sequence::Connector;
use composer::Topology;
use rand::Rng;
use kit::*;

use serde_derive::{
    Deserialize,
    Serialize,
};

use std::collections::HashMap;


fn names_of(connectors: &Vec<Connector>) -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    for c in connectors {
        xs.push(c.source.clone());
        xs.push(c.target.clone());
    }
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

    for name in &names {
        let begin = format!(
            r#"
subgraph {name}
"#
        );
        s.push_str(&begin);
        let end = format!(
            r#"
end
"#
        );
        s.push_str(&end);
    }

    for c in connectors {
        let f = format!(
            r#"
{}--{}-->{}
"#, &c.source, &c.message, &c.target
        );
        s.push_str(&f);
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

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub path: String,
    pub name: String,

}

pub fn build_tree(topologies: &HashMap<String, Topology>) -> Vec<Node> {
    let mut xs: Vec<Node> = vec![];
    let mut pindex: u8 = 1;
    for (name, topology) in topologies {
        let node = Node {
            path: format!("{}", pindex),
            name: name.to_string()
        };
        xs.push(node);

        if topology.events.len() > 0 {

            xs.push(Node { path: format!("{}.1", pindex), name: s!("events")});
            for (n, _) in &topology.events {
                let mut index: u8 = 1;
                let node = Node {
                    path: format!("{}.1.{}", pindex, index),
                    name: n.to_string()
                };
                xs.push(node);
                index += 1;
            }
        }

        if topology.routes.len() > 0 {

            xs.push(Node { path: format!("{}.2", pindex), name: s!("routes")});
            for (n, r) in &topology.routes {
                let mut index: u8 = 1;
                let node = Node {
                    path: format!("{}.2.{}", pindex, index),
                    name: format!("{} {}", r.method, r.path)
                };
                xs.push(node);
                index += 1;
            }
        }

        if topology.functions.len() > 0 {

            xs.push(Node { path: format!("{}.3", pindex), name: s!("functions")});
            for (n, f) in &topology.functions {
                let mut index: u8 = 1;
                let node = Node {
                    path: format!("{}.3.{}", pindex, index),
                    name: n.to_string()
                };
                xs.push(node);
                index += 1;
            }
        }


        pindex += 1;
    }
    xs
}
