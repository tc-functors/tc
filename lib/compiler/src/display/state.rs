use crate::Topology;

use kit as u;

pub fn display_component(topology: &Topology, component: &str) {
    if let Some(f) = &topology.flow {
        match component {
            "def" | "definition" => u::pp_json(&f.definition),
            "role" => u::pp_json(&f.role),
            _ => ()
        }
    }
}
