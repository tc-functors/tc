use tabled::{Style, Table, Tabled};

use aws::sfn;
use aws::Env;
use kit as u;

#[derive(Tabled, Clone, Debug, PartialEq)]
struct Record {
    namespace: String,
    sandbox: String,
    version: String,
    updated_at: String,
}

pub async fn list(env: &Env) {
    let client = sfn::make_client(env).await;
    let sfns = sfn::list(client).await;

    let mut rows: Vec<Record> = vec![];

    for sfn in sfns {
        let namespace = u::safe_unwrap(sfn.get("namespace"));
        if !&namespace.is_empty() {
            let row = Record {
                namespace: namespace,
                sandbox: u::safe_unwrap(sfn.get("sandbox")),
                version: u::safe_unwrap(sfn.get("version")),
                updated_at: u::safe_unwrap(sfn.get("updated_at")),
            };
            rows.push(row)
        }
    }
    let table = Table::new(rows).with(Style::psql()).to_string();
    println!("{}", table);
}
