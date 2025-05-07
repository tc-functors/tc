pub mod event;
pub mod function;
pub mod mutation;
pub mod state;
pub mod topology;
mod aws;

use colored::Colorize;
use compiler::TopologyKind;
use kit as u;
use authorizer::Auth;
use tabled::{
    Style,
    Table,
};
use topology::Record;

async fn list_sfn(auth: &Auth) {
    state::list(&auth).await
}

async fn list_fns(auth: &Auth, dir: &str, sandbox: Option<String>) {
    let fns = resolver::functions(&dir, &auth, sandbox).await;
    function::list(&auth, fns).await
}

async fn list_mutations(auth: &Auth, name: &str) {
    mutation::list(&auth, name).await
}

async fn list_layers(auth: &Auth, dir: &str, sandbox: Option<String>) {
    let fns = resolver::functions(&dir, &auth, sandbox).await;
    function::list_layers(&auth, fns).await
}

async fn list_topologies(auth: &Auth, sandbox: &str) -> Vec<Record> {
    let topologies = compiler::compile_root(&u::pwd(), false);
    topology::list(&auth, &sandbox, &topologies).await
}

pub fn pprint_topologies(records: Vec<Record>, format: &str) {
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

pub async fn lookup_version(
    auth: &Auth,
    kind: &TopologyKind,
    fqn: &str,
    sandbox: &str,
) -> Option<String> {
    let name = topology::render(fqn, sandbox);
    let tags = topology::lookup_tags(auth, kind, &name).await;
    tags.get("version").cloned()
}

async fn list_events(auth: &Auth, name: &str) {
    event::list(&auth, name).await
}

pub async fn list(auth: &Auth, sandbox: Option<String>) {
    let dir = u::pwd();
    let topology_name = compiler::topology_name(&dir);
    let sbox = resolver::maybe_sandbox(sandbox.clone());
    let name = format!("{}_{}", &topology_name, &sbox);
    let event_prefix = format!("tc-{}", &topology_name);

    println!("{}: ", "Functions".green());
    list_fns(&auth, &dir, sandbox.clone()).await;

    println!("{}: ", "Layers".blue());
    list_layers(&auth, &dir, sandbox.clone()).await;

    println!("{}: ", "Events".cyan());
    list_events(&auth, &event_prefix).await;

    println!("{}: ", "Mutations".magenta());
    list_mutations(&auth, &name).await;
}

pub async fn list_component(
    auth: &Auth,
    sandbox: Option<String>,
    component: Option<String>,
    format: Option<String>,
) {
    let dir = u::pwd();
    let component = u::maybe_string(component, "functions");
    let format = u::maybe_string(format, "table");

    if &component == "topologies" {
        let sandbox = u::maybe_string(sandbox, "stable");
        let records = list_topologies(&auth, &sandbox).await;
        pprint_topologies(records, &format);
    } else {
        let topology_name = compiler::topology_name(&dir);
        let sbox = resolver::maybe_sandbox(sandbox.clone());
        let name = format!("{}_{}", &topology_name, &sbox);
        let event_prefix = format!("tc-{}", &topology_name);

        match component.as_ref() {
            "flow" => list_sfn(&auth).await,
            "functions" => list_fns(&auth, &dir, sandbox).await,
            "layers" => list_layers(&auth, &dir, sandbox).await,
            "events" => list_events(&auth, &event_prefix).await,
            "mutations" => list_mutations(&auth, &name).await,
            _ => (),
        }
    }
}
