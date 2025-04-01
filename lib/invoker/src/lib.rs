pub mod event;
pub mod lambda;
pub mod local;
pub mod sfn;

use compiler::TopologyKind;
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
    kind: TopologyKind,
    fqn: &str,
    payload: Option<String>,
    mode: &str,
    dumb: bool
) {
    let dir = u::pwd();
    let payload = read_payload(&dir, payload);

    match kind {
        TopologyKind::Function     => lambda::invoke(env, fqn, &payload).await,
        TopologyKind::StepFunction => sfn::invoke(&env, fqn, &payload, mode, dumb).await,
        TopologyKind::Evented      => event::trigger(env, &payload).await,
        TopologyKind::Graphql      => ()
    }

}

pub async fn run_local(payload: Option<String>) {
    let dir = u::pwd();
    let payload = read_payload(&dir, payload);
    local::invoke(&payload).await;
}
