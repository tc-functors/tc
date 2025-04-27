use compiler::Topology;
use kit as u;
use tabled::Tabled;

fn cache_dir() -> String {
    String::from("/tmp/tc-resolver-cache")
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

pub fn clear() {
    u::sh(&format!("rm -rf {}", cache_dir()), &u::pwd());
}

pub fn make_key(namespace: &str, profile: &str, sandbox: &str) -> String {
    format!("{}.{}.{}", namespace, profile, sandbox)
}

pub async fn write_topology(key: &str, t: &Topology) {
    let s = serde_json::to_string(t).unwrap();
    write(key, &s).await
}

pub async fn read_topology(key: &str) -> Option<Topology> {
    if has_key(key) {
        tracing::info!("Found resolver cache: {}", key);
        let s = read(key);
        let t: Topology = serde_json::from_str(&s).unwrap();
        Some(t)
    } else {
        None
    }
}

#[derive(Tabled, Clone)]
pub struct CacheItem {
    pub namespace: String,
    pub env: String,
    pub sandbox: String,
    pub time: String,
    pub size: String,
}

pub fn list() -> Vec<CacheItem> {
    let dir = &cache_dir();
    let items = cacache::list_sync(dir);
    let mut xs: Vec<CacheItem> = vec![];
    for x in items {
        match x {
            Ok(r) => {
                if !&r.key.starts_with("functions") || !&r.key.starts_with("root") {
                    let parts: Vec<&str> = r.key.split(".").collect();
                    let namespace = parts.clone().into_iter().nth(0).unwrap_or_default();
                    let env = parts.clone().into_iter().nth(1).unwrap_or_default();
                    let sandbox = &parts.into_iter().nth(2).unwrap_or_default();

                    let c = CacheItem {
                        namespace: namespace.to_string(),
                        env: env.to_string(),
                        sandbox: sandbox.to_string(),
                        time: u::ms_to_dt(r.time as i64).to_string(),
                        size: u::file_size_human(r.size as f64),
                    };
                    xs.push(c)
                }
            }
            Err(_) => (),
        }
    }
    xs
}
