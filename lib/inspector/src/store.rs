use compiler::{Topology, Function, Event, Route};
use std::collections::HashMap;

pub async fn write_topology(key: &str, t: &Topology) {
    let s = serde_json::to_string(t).unwrap();
    cache::write(key, &s).await
}

pub async fn read_topology(key: &str) -> Option<Topology> {
    if cache::has_key(key) {
        tracing::info!("Found resolver cache: {}", key);
        let s = cache::read(key);
        let t: Topology = serde_json::from_str(&s).unwrap();
        Some(t)
    } else {
        None
    }
}

pub async fn find_all_topologies() -> HashMap<String, Topology> {
    let key = "root";
    if cache::has_key(key) {
        tracing::info!("Found cache: {}", key);
        let s = cache::read(key);
        let r: HashMap<String, Topology> = serde_json::from_str(&s).unwrap();
        r
    } else {
        HashMap::new()
    }
}

pub async fn find_topologies(root: &str, namespace: &str) -> HashMap<String, Topology> {
    let topologies = find_all_topologies().await;
    if root == namespace {
        topologies.get(root).unwrap().nodes.clone()
    } else {
        let rt = topologies.get(root);
        if let Some(t) = rt {
            t.nodes.get(namespace).unwrap().nodes.clone()
        } else {
            HashMap::new()
        }
    }
}

pub async fn find_functions(root: &str, namespace: &str) -> HashMap<String, Function> {
    let topologies = find_all_topologies().await;
    let rt = topologies.get(root);
    if root == namespace {
        match rt {
            Some(t) => t.functions.clone(),
            None => HashMap::new()
        }
    } else {
        match rt {
            Some(t) => {
                let node = t.nodes.get(namespace);
                match node {
                    Some(n) => n.functions.clone(),
                    None => HashMap::new()
                }
            },
            None => HashMap::new()
        }
    }
}

pub async fn find_events(root: &str, namespace: &str) -> HashMap<String, Event> {
    let topologies = find_all_topologies().await;
    let rt = topologies.get(root);
    if root == namespace {
        match rt {
            Some(t) => t.events.clone(),
            None => HashMap::new()
        }
    } else {
        match rt {
            Some(t) => {
                let node = t.nodes.get(namespace);
                match node {
                    Some(n) => n.events.clone(),
                    None => HashMap::new()
                }
            },
            None => HashMap::new()
        }
    }
}

pub async fn find_routes(root: &str, namespace: &str) -> HashMap<String, Route> {
    let topologies = find_all_topologies().await;
    let rt = topologies.get(root);
    if root == namespace {
        match rt {
            Some(t) => t.routes.clone(),
            None => HashMap::new()
        }
    } else {
        match rt {
            Some(t) => {
                let node = t.nodes.get(namespace);
                match node {
                    Some(n) => n.routes.clone(),
                    None => HashMap::new()
                }
            },
            None => HashMap::new()
        }
    }
}
// singular

pub async fn find_topology(root: &str, namespace: &str) -> Option<Topology> {
    let topologies = find_all_topologies().await;
    if root == namespace {
        topologies.get(root).cloned()
    } else {
        let rt = topologies.get(root);
        if let Some(t) = rt {
            t.nodes.get(namespace).cloned()
        } else {
            None
        }
    }
}



pub async fn find_function(root: &str, namespace: &str, id: &str) -> Option<Function> {
    let topologies = find_all_topologies().await;
    if root == namespace {
        let rt = topologies.get(root).unwrap();
        tracing::debug!("{:?}", rt.functions);
        rt.functions.get(id).cloned()
    } else {
        let rt = topologies.get(root);
        if let Some(t) = rt {
            t.functions.get(id).cloned()
        } else {
            None
        }
    }
}


// resolver


pub async fn find_resolved_topology(namespace: &str, sandbox: &str, env: &str) -> Option<Topology> {
    None
}
