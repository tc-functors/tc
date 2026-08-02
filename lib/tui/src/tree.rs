use composer::Topology;
use tui_tree_widget::TreeItem;


pub fn make_events<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, _events) in &topology.events {
        let item = TreeItem::new_leaf(name.as_str(), name.as_str());
            xs.push(item);
    }

    TreeItem::new("events", "Events", xs)
        .expect("all item identifiers are unique")
}

pub fn make_routes<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, route) in &topology.routes {
        let item = TreeItem::new_leaf(name.as_str(), route.path.as_str());
            xs.push(item);
    }

    TreeItem::new("routes", "Routes", xs)
        .expect("all item identifiers are unique")
}


pub fn make_functions<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, _events) in &topology.functions {
        let item = TreeItem::new_leaf(name.as_str(), name.as_str());
            xs.push(item);
    }

    TreeItem::new("functions", "Functions", xs)
        .expect("all item identifiers are unique")
}

pub fn make_mutations<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    if let Some(mutations) = topology.mutations.get("default") {
        for (name, _m) in &mutations.resolvers {
            let item = TreeItem::new_leaf(name.as_str(), name.as_str());
            xs.push(item);
        }

        TreeItem::new("mutations", "Mutations", xs)
            .expect("all item identifiers are unique")
    } else {
        TreeItem::new("mutations", "Mutations", vec![])
            .expect("all item identifiers are unique")
    }
}
