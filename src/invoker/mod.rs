pub mod event;
pub mod lambda;
pub mod local;
pub mod sfn;

use crate::compiler;
use crate::resolver;
use aws::Env;
use kit as u;

fn read_payload(dir: &str, s: Option<String>) -> String {
    match s {
        Some(p) => {
            if p.ends_with(".json") && u::file_exists(&p) {
                u::slurp(&p)
            } else {
                p
            }
        }
        None => {
            let f = format!("{}/payload.json", dir);
            if u::file_exists(&f) {
                u::slurp(&f)
            } else {
                u::read_stdin()
            }
        }
    }
}

pub async fn invoke(
    env: &Env,
    sandbox: &str,
    kind: &str,
    name: Option<String>,
    payload: Option<String>,
    mode: Option<String>,
    dumb: bool,
) {
    let dir = u::pwd();
    let payload = read_payload(&dir, payload);
    let inferred_name = resolver::name_of(&dir, sandbox, kind).await;
    let name = match inferred_name {
        Some(n) => u::maybe_string(name, &n),
        None => u::maybe_string(name, "default"),
    };

    match kind.as_ref() {
        "lambda" | "function" => lambda::invoke(env, &name, &payload).await,
        "event" => event::trigger(env, &payload).await,
        "step-function" | "state-machine" => {
            let inferred_mode = compiler::topology_mode(&dir);
            let mode = u::maybe_string(mode, inferred_mode);
            sfn::invoke(&env, &name, &mode, &payload, dumb).await;
        }
        _ => println!(""),
    }
}

pub async fn run_local(payload: Option<String>) {
    let dir = u::pwd();
    let payload = read_payload(&dir, payload);
    local::invoke(&payload).await;
}
