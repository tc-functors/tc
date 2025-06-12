use kit as u;
use serde_json::Value;

pub fn read(dir: &str, def: Value) -> Value {
    match def.as_str() {
        Some(p) => {
            let path = format!("{}/{}", dir, &p);
            if (path.ends_with(".json") || path.ends_with(".yml")) && u::file_exists(&path) {
                let data = u::slurp(&path);
                u::json_value(&data)
            } else {
                def
            }
        }
        None => def,
    }
}
