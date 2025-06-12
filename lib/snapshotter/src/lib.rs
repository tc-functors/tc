
use kit as u;
use authorizer::Auth;
mod aws;
mod versions;
mod topology;

use versions::Record;
use topology::Topology;

use tabled::{
    settings::Style,
    Table,
};


pub async fn snapshot(auth: &Auth, dir: &str, sandbox: &str) -> Vec<Record> {
    versions::find(auth, dir, sandbox).await
}

pub fn pretty_print(records: Vec<Record>, format: &str) {
    match format {
        "table" => {
            let table = Table::new(records).with(Style::psql()).to_string();
            println!("{}", table);
        }
        "json" => {
            let s = u::pretty_json(records);
            println!("{}", &s);
        }
        _ => (),
    }
}

pub async fn snapshot_profiles(dir: &str, sandbox: &str, profiles: Vec<String>) {
    versions::find_by_profiles(dir, sandbox, profiles).await;
}

pub async fn snapshot_topology(auth: &Auth, dir: &str, sandbox: &str) {
    let t = Topology::new(auth, dir, sandbox).await;
    let s = u::pretty_json(&t);
    println!("{}", &s);
}

pub async fn snapshot_entity(_auth: &Auth, _dir: &str, _sandbox: &str, _entity: &str) {
    println!("Nothing yet")
}
