use crate::Topology;

use kit as u;

pub fn display_component(topology: &Topology, component: &str) {
    let events = &topology.events;
    match events.get(component) {
        Some(c) => u::pp_json(&c),
        None => println!("No event found")
    }
}
