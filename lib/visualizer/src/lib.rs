mod node;
mod digraph;
mod system;

use system::Node;
use compiler::TopologySpec;
use composer::Topology;
use composer::sequence;
use composer::sequence::Connector;
use kit as u;
use std::collections::HashMap;

use serde_derive::{
    Deserialize,
    Serialize,
};

use base64::{
    Engine as _,
    engine::general_purpose,
};

pub fn visualize_node(topology: &Topology, theme: &str) {
    println!("Generating SVG...");
    let html = node::generate(topology, theme);
    let dir = u::pwd();
    let path = format!("{}/flow.html", &dir);
    u::write_str(&path, &html);
    println!("Opening {}", &path);
    open::that(path).unwrap();
}

pub fn visualize_root(_topologies: HashMap<String, Topology>, _theme: &str) {
    // let html = evented::generate(&topologies, theme);
    // let dir = u::pwd();
    // let path = format!("{}/root.html", &dir);
    // u::write_str(&path, &html);
    // println!("Opening {}", &path);
    // open::that(path).unwrap();
    println!("Not implemented")
}

pub fn visualize(dir: &str, recursive: bool, theme: &str, dirs: Vec<String>) {
    let is_root = composer::is_root_dir(&dir);
    if !dirs.is_empty() {
        let tps = composer::compose_dirs(dirs);
        visualize_root(tps, theme)
    } else if is_root || recursive {
        let tps = composer::compose_root(&dir, recursive);
        visualize_root(tps, theme)
    } else {
        let topology = composer::compose(&dir, false);
        visualize_node(&topology, theme);
    }
}

pub fn gen_mermaid(topology: &Topology) -> String {
    node::generate_diagram(topology, "light")
}

pub fn gen_dot(topology: &Topology) -> String {
    node::generate_dot(topology)
}

// actual

pub fn gen_topology(topology: &Topology) -> (String, String) {
    let mermaid_dia = node::generate_diagram(topology, "light");
    let dot_dia = node::generate_dot(topology);
    (mermaid_dia, dot_dia)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct System {
    pub sequence: String,
    pub flow: String,
    pub namespaces: Vec<String>,
    pub definition: Vec<String>,
}

pub fn gen_system(cspecs: HashMap<String, Vec<String>>) -> HashMap<String, System> {
    let sequence = sequence::make_seq(&cspecs);
    let mut h: HashMap<String, System> = HashMap::new();
    for (name, connectors) in sequence {
        let st = cspecs.get(&name).unwrap();
        //st.retain(|s| !s.is_empty());
        let seq_dia = system::gen_sequence(&connectors);
        let flow_dia = system::gen_flow(&connectors);
        let namespaces = system::names_of(&connectors);
        let system = System {
            sequence: general_purpose::STANDARD.encode(&seq_dia),
            namespaces: system::names_of(&connectors),
            flow: general_purpose::STANDARD.encode(&flow_dia),
            definition: st.to_vec()
        };
        h.insert(name, system);
    }
    h
}

pub fn gen_system_from_connectors(cxs_map: &HashMap<String, Vec<Connector>>) -> HashMap<String, System> {
    let mut h: HashMap<String, System> = HashMap::new();
    for (name, connectors) in cxs_map {
        let seq_dia = system::gen_sequence(connectors);
        let flow_dia = system::gen_flow(connectors);
        let mut xs: Vec<String> = vec![];
        for c in connectors {
            let p = format!(r#"{} -> {} -> {}"#, &c.source, &c.message, &c.target);
            xs.push(p);
        }
        let namespaces = system::names_of(&connectors);
        let system = System {
            sequence: general_purpose::STANDARD.encode(&seq_dia),
            namespaces: namespaces,
            flow: general_purpose::STANDARD.encode(&flow_dia),
            definition: xs
        };
        h.insert(name.to_string(), system);
    }
    h
}

pub fn gen_system_tree(tspecs: &HashMap<String, TopologySpec>) -> String {
    let tree = system::build_shallow_tree(tspecs);
    serde_json::to_string(&tree).unwrap()
}
