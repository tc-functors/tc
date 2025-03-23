use compiler::Topology;

pub fn make_key(namespace: &str, profile: &str, sandbox: &str) -> String {
    format!("{}.{}.{}", namespace, profile, sandbox)
}

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
