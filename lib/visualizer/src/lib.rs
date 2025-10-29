mod node;
mod overview;
mod digraph;

use composer::Topology;
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
    let html = overview::generate(&topologies, theme);
    let dir = u::pwd();
    let path = format!("{}/root.html", &dir);
    u::write_str(&path, &html);
    println!("Opening {}", &path);
    open::that(path).unwrap();
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

pub fn gen_root_mermaid(topologies: &HashMap<String, Topology>) -> String {
    overview::generate_diagram(topologies, "light")
}
