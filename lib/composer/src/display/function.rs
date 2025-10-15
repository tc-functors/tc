use crate::Topology;
use colored::Colorize;
use kit as u;
use kit::*;
use ptree::{
    builder::TreeBuilder,
    item::StringItem,
};

fn as_uri(s: &str) -> String {
    if s.starts_with("/") {
        u::gdir(&s)
    } else {
        s.to_string()
    }
}

pub fn build_tree(topology: &Topology) -> StringItem {
    let mut t = TreeBuilder::new(s!(topology.namespace.blue()));

    for (name, f) in &topology.functions {
        t.begin_child(s!(name.green()));
        t.add_empty_child(s!(&f.fqn));
        t.add_empty_child(format!("lang: {}", f.runtime.lang.to_str()));
        t.add_empty_child(format!("role: {}", f.runtime.role.name.to_string()));
        t.add_empty_child(format!("uri: {}", as_uri(&f.runtime.uri)));
        t.add_empty_child(format!("build: {}", f.build.kind.to_str()));
        t.end_child();
    }

    for (_, node) in &topology.nodes {
        t.begin_child(s!(&node.namespace.green()));
        for (_, f) in &node.functions {
            t.begin_child(s!(&f.fqn));
            t.add_empty_child(format!("lang: {}", f.runtime.lang.to_str()));
            t.add_empty_child(format!("role: {}", f.runtime.role.name.to_string()));
            t.add_empty_child(format!("uri: {}", as_uri(&f.runtime.uri)));
            t.add_empty_child(format!("build: {}", f.build.kind.to_str()));
            t.end_child();
        }
        t.end_child();
    }
    t.build()
}

pub fn display_component(topology: &Topology, component: &str) {
    let xs: Vec<&str> = component.split("/").collect();
    let functions = &topology.functions;

    if xs.len() == 2 {
        let fn_str = u::nth(xs.clone(), 0);
        let part = u::nth(xs, 1);
        let function = functions.get(&fn_str);
        match function {
            Some(f) => match part.as_ref() {
                "build" => u::pp_json(&f.build),
                "vars" => u::pp_json(&f.runtime.environment),
                "tags" => u::pp_json(&f.runtime.tags),
                "role" => u::pp_json(&f.runtime.role),
                _ => (),
            },
            None => println!("Function not found"),
        }
    } else if xs.len() == 1 {
        let fn_str = u::nth(xs, 0);
        let function = functions.get(&fn_str);
        if let Some(f) = function {
            u::pp_json(f)
        }
    }
}
