use compiler::{Channel, Event, Function, Route, Topology};
use std::collections::HashMap;

fn cache_dir() -> String {
    String::from("/tmp/tc-inspector-cache")
}

pub async fn write(key: &str, value: &str) {
    let _ = cacache::write(&cache_dir(), key, value.as_bytes()).await;
}

pub fn read(key: &str) -> String {
    let data = cacache::read_sync(&cache_dir(), key);
    match data {
        Ok(buf) => String::from_utf8_lossy(&buf).to_string(),
        Err(_) => panic!("no cache found"),
    }
}

pub fn has_key(key: &str) -> bool {
    let data = cacache::read_sync(&cache_dir(), key);
    match data {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn _write_topology(key: &str, t: &Topology) {
    let s = serde_json::to_string(t).unwrap();
    write(key, &s).await
}

pub async fn find_all_topologies() -> HashMap<String, Topology> {
    let key = "root";
    if has_key(key) {
        tracing::info!("Found cache: {}", key);
        let s = read(key);
        let r: HashMap<String, Topology> = serde_json::from_str(&s).unwrap();
        r
    } else {
        HashMap::new()
    }
}

pub async fn find_functions(root: &str, namespace: &str) -> HashMap<String, Function> {
    let topologies = find_all_topologies().await;
    let rt = topologies.get(root);
    if root == namespace {
        match rt {
            Some(t) => t.functions.clone(),
            None => HashMap::new(),
        }
    } else {
        match rt {
            Some(t) => {
                let node = t.nodes.get(namespace);
                match node {
                    Some(n) => n.functions.clone(),
                    None => HashMap::new(),
                }
            }
            None => HashMap::new(),
        }
    }
}

pub async fn find_all_events() -> HashMap<String, Event> {
    let topologies = find_all_topologies().await;
    let mut h: HashMap<String, Event> = HashMap::new();
    for (_, node) in topologies {
        h.extend(node.events);
        for (_, n) in node.nodes {
            h.extend(n.events);
        }
    }
    h
}

// by namespace

pub async fn find_events(root: &str, namespace: &str) -> HashMap<String, Event> {
    let topologies = find_all_topologies().await;
    let rt = topologies.get(root);
    if root == namespace {
        match rt {
            Some(t) => t.events.clone(),
            None => HashMap::new(),
        }
    } else {
        match rt {
            Some(t) => {
                let node = t.nodes.get(namespace);
                match node {
                    Some(n) => n.events.clone(),
                    None => HashMap::new(),
                }
            }
            None => HashMap::new(),
        }
    }
}

pub async fn _find_channels(root: &str, namespace: &str) -> HashMap<String, Channel> {
    let topologies = find_all_topologies().await;
    let rt = topologies.get(root);
    if root == namespace {
        match rt {
            Some(t) => t.channels.clone(),
            None => HashMap::new(),
        }
    } else {
        match rt {
            Some(t) => {
                let node = t.nodes.get(namespace);
                match node {
                    Some(n) => n.channels.clone(),
                    None => HashMap::new(),
                }
            }
            None => HashMap::new(),
        }
    }
}

pub async fn find_routes(root: &str, namespace: &str) -> HashMap<String, Route> {
    let topologies = find_all_topologies().await;
    let rt = topologies.get(root);
    if root == namespace {
        match rt {
            Some(t) => t.routes.clone(),
            None => HashMap::new(),
        }
    } else {
        match rt {
            Some(t) => {
                let node = t.nodes.get(namespace);
                match node {
                    Some(n) => n.routes.clone(),
                    None => HashMap::new(),
                }
            }
            None => HashMap::new(),
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

pub async fn find_root_namespaces() -> Vec<String> {
    let ts = find_all_topologies().await;
    let mut xs: Vec<String> = vec![];
    for (_, t) in ts {
        if !t.events.is_empty() {
            xs.push(t.namespace)
        }
    }
    xs
}

pub async fn init() {
    let topologies = compiler::compile_root(&kit::pwd(), true);
    write("root", &serde_json::to_string(&topologies).unwrap()).await;
}
