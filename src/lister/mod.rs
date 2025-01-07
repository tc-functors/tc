pub mod event;
pub mod function;
pub mod layer;
pub mod mutation;
pub mod sfn;
pub mod topology;

use crate::compiler;
use crate::resolver;
use aws::Env;
use colored::Colorize;
use kit as u;

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

async fn list_topologies(env: &Env, sandbox: Option<String>, format: &str) {
    let sandbox = u::maybe_string(sandbox, "stable");
    let topologies = compiler::list_topologies();
    let mut names: Vec<String> = vec![];
    for (_, spec) in topologies {
        let name = if spec.hyphenated_names {
            format!("{}-{}", &spec.name, &sandbox)
        } else {
            format!("{}_{}", &spec.name, &sandbox)
        };
        names.push(name);
    }
    topology::list(&env, names, format).await
}

async fn list_events(env: &Env, name: &str) {
    event::list(&env, name).await
}

pub async fn list(env: &Env, sandbox: Option<String>) {
    let dir = u::pwd();
    let topology_name = compiler::topology_name(&dir);
    let sbox = resolver::as_sandbox(sandbox.clone());
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
        list_topologies(&env, sandbox, &format).await;
    } else {
        let topology_name = compiler::topology_name(&dir);
        let sbox = resolver::as_sandbox(sandbox.clone());
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
