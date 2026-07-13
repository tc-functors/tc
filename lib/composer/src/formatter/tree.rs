use crate::Topology;
use ptree::TreeBuilder;
use std::collections::HashMap;

use colored::Colorize;
use kit as u;
use kit::s;

fn as_uri(s: &str) -> String {
    if s.starts_with("/") {
        u::gdir(&s)
    } else {
        s.to_string()
    }
}

pub fn pprint(topology: &Topology) {
    let Topology {
        namespace,
        functions,
        events,
        routes,
        ..
    } = topology;
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

pub fn pprint_recursive(topologies: &HashMap<String, Topology>) {
    let mut t = TreeBuilder::new(String::from(""));

    for (name, topology) in topologies {
        let v = if &topology.version == "0.0.1" {
            ""
        } else {
            &topology.version.green()
        };
        let nt = format!(
            "{} (v:{} f:{}, e:{})",
            name.cyan(),
            v.green(),
            &topology.functions.len(),
            &topology.events.len()
        );
        t.begin_child(nt);
        for (n, node) in &topology.nodes {
            let v = if &node.version == "0.0.1" {
                ""
            } else {
                &node.version.green()
            };
            let tt = format!(
                "{} (v:{} f:{} e:{}) ",
                n.blue(),
                &v,
                &node.functions.len(),
                &node.events.len()
            );
            t.add_empty_child(tt);
        }
        t.end_child();
    }

    let tree = t.build();
    kit::print_tree(tree);
}
