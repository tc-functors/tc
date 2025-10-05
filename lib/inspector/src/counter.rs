use crate::Store;
use composer::Topology;

pub struct Counter {
    pub functions: usize,
    pub events: usize,
    pub routes: usize,
    pub mutations: usize,
    pub queues: usize,
    pub channels: usize,
    pub states: usize,
    pub pages: usize,
}

pub async fn count_of(store: &Store, root: &str, namespace: &str) -> Counter {
    let f = store.find_topology(root, namespace).await;
    if let Some(t) = f {
        Counter {
            functions: t.functions.len(),
            events: t.events.len(),
            routes: t.routes.len(),
            mutations: t.mutations.len(),
            queues: t.queues.len(),
            channels: t.channels.len(),
            pages: t.pages.len(),
            states: 0,
        }
    } else {
        Counter {
            functions: 0,
            events: 0,
            routes: 0,
            mutations: 0,
            queues: 0,
            channels: 0,
            states: 0,
            pages: 0,
        }
    }
}

fn count_functions(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        c = c + node.functions.len();
        for (_, n) in &node.nodes {
            c = c + n.functions.len();
        }
    }
    c
}

fn count_events(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        c = c + node.events.len();
        for (_, n) in &node.nodes {
            c = c + n.events.len();
        }
    }
    c
}

fn count_routes(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        c = c + node.routes.len();
        for (_, n) in &node.nodes {
            c = c + n.routes.len();
        }
    }
    c
}

fn count_mutations(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        let m = node.mutations.get("default");
        if let Some(mo) = m {
            c = c + mo.resolvers.len();
        }
        for (_, n) in &node.nodes {
            let m = n.mutations.get("default");
            if let Some(mo) = m {
                c = c + mo.resolvers.len();
            }
        }
    }
    c
}

fn count_queues(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        c = c + node.queues.len();
        for (_, n) in &node.nodes {
            c = c + n.queues.len();
        }
    }
    c
}

fn count_channels(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        c = c + node.channels.len();
        for (_, n) in &node.nodes {
            c = c + n.channels.len();
        }
    }
    c
}

fn count_pages(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        c = c + node.pages.len();
        for (_, n) in &node.nodes {
            c = c + n.pages.len();
        }
    }
    c
}

fn count_states(xs: &Vec<Topology>) -> usize {
    let mut c: usize = 0;
    for node in xs {
        if let Some(_f) = &node.flow {
            c = c + 1;
        }
        for (_, n) in &node.nodes {
            if let Some(_f) = &n.flow {
                c = c + 1;
            }
        }
    }
    c
}

pub async fn count_all(topologies: &Vec<Topology>) -> Counter {
    Counter {
        functions: count_functions(&topologies),
        events: count_events(&topologies),
        routes: count_routes(&topologies),
        mutations: count_mutations(&topologies),
        queues: count_queues(&topologies),
        channels: count_channels(&topologies),
        states: count_states(&topologies),
        pages: count_pages(&topologies),
    }
}
