mod node;
mod digraph;
mod system;

use composer::Topology;
use composer::sequence;
use composer::sequence::Connector;
use kit as u;
use std::collections::HashMap;

pub fn visualize_node(topology: &Topology, theme: &str) {
    println!("Generating SVG...");
    let html = node::generate(topology, theme);
    let dir = u::pwd();
    let path = format!("{}/flow.html", &dir);
    u::write_str(&path, &html);
    println!("Opening {}", &path);
    open::that(path).unwrap();
}

pub fn visualize_root(topologies: HashMap<String, Topology>, theme: &str) {
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

pub fn gen_system(cspec: Vec<String>) -> (String, String, Vec<String>) {
    let mut st = cspec;
    st.retain(|s| !s.is_empty());
    let sequence = sequence::make_seq(&st);
    let seq_dia = system::gen_sequence(&sequence);
    let flow_dia = system::gen_flow(&sequence);
    (seq_dia, flow_dia, st)
}

pub fn gen_system_from_connectors(connectors: &Vec<Connector>) -> (String, String, Vec<String>) {
    let seq_dia = system::gen_sequence(connectors);
    let flow_dia = system::gen_flow(connectors);
    let mut xs: Vec<String> = vec![];
    for c in connectors {
        let p = format!(r#"{} -> {} -> {}"#, &c.source, &c.message, &c.target);
        xs.push(p);
    }
    (seq_dia, flow_dia, xs)
}
