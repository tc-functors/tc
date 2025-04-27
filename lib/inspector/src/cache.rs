use compiler::{
    Event,
    Function,
    Route,
    Topology,
};
use serde_derive::{
    Deserialize,
    Serialize,
};
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

pub async fn find_layers() -> Vec<String> {
    let mut xs: Vec<String> = vec![];
    let topologies = find_all_topologies().await;
    for (_, node) in topologies {
        for (_, f) in node.functions {
            xs.extend(f.runtime.layers)
        }
        for (_, n) in node.nodes {
            for (_, f) in n.functions {
                xs.extend(f.runtime.layers)
            }
        }
    }
    xs.sort();
    xs.dedup();
    xs
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub dev: i64,
    pub stable: i64,
}

pub async fn save_resolved_layers(layers: Vec<Layer>) {
    write("resolved_layers", &serde_json::to_string(&layers).unwrap()).await;
}

pub async fn find_resolved_layers() -> Vec<Layer> {
    let key = "resolved_layers";
    if has_key(key) {
        tracing::info!("Found cache: {}", key);
        let s = read(key);
        let r: Vec<Layer> = serde_json::from_str(&s).unwrap();
        r
    } else {
        vec![]
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

pub type Versions = HashMap<String, HashMap<String, String>>;

pub async fn save_versions(vers: Versions) {
    write("versions", &serde_json::to_string(&vers).unwrap()).await;
}

pub async fn find_versions() -> Option<Versions> {
    let key = "versions";
    if has_key(key) {
        tracing::info!("Found cache: {}", key);
        let s = read(key);
        let r: Versions = serde_json::from_str(&s).unwrap();
        Some(r)
    } else {
        None
    }
}
