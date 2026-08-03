use composer::Topology;
use tui_tree_widget::TreeItem;


pub fn make_events<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, _events) in &topology.events {
        let item = TreeItem::new_leaf(name.as_str(), name.as_str());
            xs.push(item);
    }
    let count = &topology.events.len();

    TreeItem::new("events", format!("Events ({})", &count), xs)
        .expect("all item identifiers are unique")
}

pub fn make_routes<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, route) in &topology.routes {
        let item = TreeItem::new_leaf(name.as_str(), route.path.as_str());
            xs.push(item);
    }

    let count = &topology.routes.len();

    TreeItem::new("routes", format!("Routes ({})", &count), xs)
        .expect("all item identifiers are unique")
}


pub fn make_functions<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];


    for (name, _f) in &topology.functions {
        let build = TreeItem::new_leaf("build", "build");
        let runtime = TreeItem::new_leaf("runtime", "runtime");
        let role = TreeItem::new_leaf("role", "permissions");
        let env = TreeItem::new_leaf("environment", "environment");

        let components = vec![build, runtime, env, role];
        let item = TreeItem::new(name.as_str(), name.as_str(), components).expect("all item identifiers are unique");
        xs.push(item);
    }

    let count = &topology.functions.len();
    TreeItem::new("functions", format!("Functions ({})", &count), xs)
        .expect("all item identifiers are unique")
}

pub fn make_mutations<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    if let Some(mutations) = topology.mutations.get("default") {
        for (name, _m) in &mutations.resolvers {
            let item = TreeItem::new_leaf(name.as_str(), name.as_str());
            xs.push(item);
        }

        let count = &mutations.resolvers.len();
        TreeItem::new("mutations", format!("Mutations ({})", &count), xs)
            .expect("all item identifiers are unique")
    } else {
        TreeItem::new("mutations", "Mutations (0)", vec![])
            .expect("all item identifiers are unique")
    }
}


pub fn make_pages<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, _m) in &topology.pages {
        let item = TreeItem::new_leaf(name.as_str(), name.as_str());
        xs.push(item);
    }

    let count = &topology.pages.len();
    TreeItem::new("pages", format!("Pages ({})", &count), xs)
        .expect("all item identifiers are unique")
}

pub fn make_channels<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, _m) in &topology.channels {
        let item = TreeItem::new_leaf(name.as_str(), name.as_str());
        xs.push(item);
    }

    let count = &topology.channels.len();
    TreeItem::new("channels", format!("Channels ({})", &count), xs)
        .expect("all item identifiers are unique")
}

pub fn make_queues<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, _m) in &topology.queues {
        let item = TreeItem::new_leaf(name.as_str(), name.as_str());
        xs.push(item);
    }

    let count = &topology.queues.len();
    TreeItem::new("queues", format!("Queues ({})", &count), xs)
        .expect("all item identifiers are unique")
}


pub fn make_roles<'a>(topology: &'a Topology) -> TreeItem<'a, &'a str> {
    let mut xs: Vec<TreeItem<'a, &'a str>> = vec![];

    for (name, _m) in &topology.roles {
        let item = TreeItem::new_leaf(name.as_str(), name.as_str());
        xs.push(item);
    }

    let count = &topology.roles.len();
    TreeItem::new("roles", format!("Roles ({})", &count), xs)
        .expect("all item identifiers are unique")
}
