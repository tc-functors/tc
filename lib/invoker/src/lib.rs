mod aws;
mod event;
mod function;
mod state;
use authorizer::Auth;
use composer::TopologyKind;
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
    auth: &Auth,
    kind: TopologyKind,
    fqn: &str,
    payload: Option<String>,
    mode: &str,
    dumb: bool,
) {
    let dir = u::pwd();
    let payload = read_payload(&dir, payload);

    match kind {
        TopologyKind::Function => function::invoke(auth, fqn, &payload).await,
        TopologyKind::StepFunction => state::invoke(auth, fqn, &payload, mode, dumb).await,
        TopologyKind::Evented => event::trigger(auth, &payload).await,
        TopologyKind::Graphql => (),
    }
}

pub async fn run_local(payload: Option<String>) {
    let dir = u::pwd();
    let payload = read_payload(&dir, payload);
    function::invoke_local(&payload).await;
}
