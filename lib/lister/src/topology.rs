use serde_derive::{Deserialize, Serialize};
use tabled::{Style, Table, Tabled};

use aws::sfn;
use aws::Env;
use kit as u;
use std::collections::HashMap;

#[derive(Tabled, Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    namespace: String,
    sandbox: String,
    version: String,
    frozen: String,
    updated_at: String,
}

async fn lookup_tags(env: &Env, name: &str) -> HashMap<String, String> {
    let client = sfn::make_client(env).await;
    let states_arn = env.sfn_arn(&name);
    sfn::list_tags(&client, &states_arn).await.unwrap()
}

pub async fn list(env: &Env, names: Vec<String>, format: &str) {
    let mut rows: Vec<Record> = vec![];
    for name in names {
        let tags = lookup_tags(env, &name).await;
        let namespace = u::safe_unwrap(tags.get("namespace"));
        if !&namespace.is_empty() {
            let version = u::safe_unwrap(tags.get("version"));
            if version != "0.0.1" {
                let row = Record {
                    namespace: namespace,
                    sandbox: u::safe_unwrap(tags.get("sandbox")),
                    version: version,
                    frozen: u::safe_unwrap(tags.get("freeze")),
                    updated_at: u::safe_unwrap(tags.get("updated_at")),
                };
                rows.push(row)
            }
        }
    }
    match format {
        "table" => {
            let table = Table::new(rows).with(Style::psql()).to_string();
            println!("{}", table);
        }
        "json" => {
            let s = u::pretty_json(rows);
            println!("{}", &s);
        }
        _ => (),
    }
}
