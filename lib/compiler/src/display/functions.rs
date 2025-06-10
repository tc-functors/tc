use ptree::{
    builder::TreeBuilder,
    item::StringItem,
};
use colored::Colorize;
use crate::Topology;
use kit::*;

pub fn build_tree(topology: &Topology) -> StringItem {
    let mut t = TreeBuilder::new(s!(topology.namespace.blue()));

    for (_, f) in &topology.functions {
        t.begin_child(s!(f.name.green()));
        t.add_empty_child(f.runtime.lang.to_str());
        t.add_empty_child(f.runtime.role.path.to_string());
        t.add_empty_child(f.dir.to_string());
        t.add_empty_child(f.build.kind.to_str());
        t.end_child();
    }

    for (_, node) in &topology.nodes {
        t.begin_child(s!(&node.namespace.green()));
        for (_, f) in &node.functions {
            t.begin_child(s!(&f.fqn));
            t.add_empty_child(f.runtime.lang.to_str());
            t.add_empty_child(f.runtime.role.path.to_string());
            t.add_empty_child(f.dir.to_string());
            t.add_empty_child(f.build.kind.to_str());
            t.end_child();
        }
        t.end_child();
    }
    t.build()
}
