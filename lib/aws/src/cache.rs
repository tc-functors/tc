fn cache_dir() -> String {
    String::from("/tmp/tc-cache")
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
