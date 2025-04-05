pub mod event;
pub mod function;
pub mod layer;
pub mod mutation;
pub mod sfn;
pub mod topology;

use aws::Env;
use colored::Colorize;
use kit as u;
use topology::Record;

use compiler::TopologyKind;

use tabled::{Style, Table};

async fn list_sfn(env: &Env) {
    sfn::list(&env).await
}

async fn list_fns(env: &Env, dir: &str, sandbox: Option<String>) {
    let fns = resolver::functions(&dir, &env, sandbox).await;
    function::list(&env, fns).await
}

async fn list_mutations(env: &Env, name: &str) {
    mutation::list(&env, name).await
}

async fn list_layers(env: &Env, dir: &str, sandbox: Option<String>) {
    let fns = resolver::functions(&dir, &env, sandbox).await;
    layer::list(&env, fns).await
}

async fn list_topologies(env: &Env, sandbox: &str) -> Vec<Record> {
    let topologies = compiler::compile_root(&u::pwd(), false);
    topology::list(&env, &sandbox, &topologies).await
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

pub async fn lookup_version(env: &Env, kind: &TopologyKind, fqn: &str, sandbox: &str) -> Option<String> {
    let name = topology::render(fqn, sandbox);
    let tags = topology::lookup_tags(env, kind, &name).await;
    tags.get("version").cloned()
}

async fn list_events(env: &Env, name: &str) {
    event::list(&env, name).await
}

pub async fn list(env: &Env, sandbox: Option<String>) {
    let dir = u::pwd();
    let topology_name = compiler::topology_name(&dir);
    let sbox = resolver::maybe_sandbox(sandbox.clone());
    let name = format!("{}_{}", &topology_name, &sbox);
    let event_prefix = format!("tc-{}", &topology_name);

    println!("{}: ", "Functions".green());
    list_fns(&env, &dir, sandbox.clone()).await;

    println!("{}: ", "Layers".blue());
    list_layers(&env, &dir, sandbox.clone()).await;

    println!("{}: ", "Events".cyan());
    list_events(&env, &event_prefix).await;

    println!("{}: ", "Mutations".magenta());
    list_mutations(&env, &name).await;
}

pub async fn list_component(
    env: &Env,
    sandbox: Option<String>,
    component: Option<String>,
    format: Option<String>,
) {
    let dir = u::pwd();
    let component = u::maybe_string(component, "functions");
    let format = u::maybe_string(format, "table");

    if &component == "topologies" {
        let sandbox = u::maybe_string(sandbox, "stable");
        let records = list_topologies(&env, &sandbox).await;
        pprint_topologies(records, &format);
    } else {
        let topology_name = compiler::topology_name(&dir);
        let sbox = resolver::maybe_sandbox(sandbox.clone());
        let name = format!("{}_{}", &topology_name, &sbox);
        let event_prefix = format!("tc-{}", &topology_name);

        match component.as_ref() {
            "flow" => list_sfn(&env).await,
            "functions" => list_fns(&env, &dir, sandbox).await,
            "layers" => list_layers(&env, &dir, sandbox).await,
            "events" => list_events(&env, &event_prefix).await,
            "mutations" => list_mutations(&env, &name).await,
            _ => (),
        }
    }
}
