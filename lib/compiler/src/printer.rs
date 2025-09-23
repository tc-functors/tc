use kit as u;
use kit::*;
use serde_derive::Serialize;
use std::collections::HashMap;
use tabled::{
    Style,
    Table,
    Tabled,
};

#[derive(Tabled, Clone, Debug, Serialize)]
struct Version {
    namespace: String,
    version: String,
}

pub fn print_versions(versions: HashMap<String, String>, format: &str) {
    let mut xs: Vec<Version> = vec![];
    for (namespace, version) in versions {
        let v = Version {
            namespace: s!(namespace),
            version: s!(version),
        };
        xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
        xs.reverse();
        xs.push(v)
    }
    match format {
        "table" => {
            let table = Table::new(xs).with(Style::psql()).to_string();
            println!("{}", table);
        }
        "json" => u::pp_json(&xs),
        &_ => todo!(),
    }
}
