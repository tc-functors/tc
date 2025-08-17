pub mod aws;
mod event;
mod function;
mod state;
mod repl;
use authorizer::Auth;
use composer::{Entity, Topology};
use kit as u;


async fn read_uri(auth: &Auth, uri: &str) -> String {
    let client = aws::s3::make_client(auth).await;
    let (bucket, key) = aws::s3::parts_of(uri);
    aws::s3::get_str(&client, &bucket, &key).await
}

pub fn read_payload_local(payload: Option<String>) -> String {
    if let Some(p) = payload {
        if p.ends_with(".json") && u::file_exists(&p) {
            u::slurp(&p)
        } else {
            p
        }
    } else {
        panic!("No payload data found")
    }
}

pub async fn read_payload(auth: &Auth, dir: &str, s: Option<String>) -> String {
    match s {
        Some(p) => {
            if p.starts_with("s3://") {
                read_uri(auth, &p).await
            } else {
                if p.ends_with(".json") && u::file_exists(&p) {
                    u::slurp(&p)
                } else {
                    p
                }
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

async fn invoke_component(auth: &Auth, topology: &Topology, entity: Entity, component: &str, payload: &str) {
    match entity {
        Entity::Function => {
            let functions = &topology.functions;
            if let Some(f) = functions.get(component) {
                function::invoke(auth, &f.fqn, payload).await;
            } else {
                println!("Function not found")
            }
        }
        Entity::Event => {
            let events = &topology.events;
            if let Some(e) = events.get(component) {
                let detail_type = &e.pattern.detail_type.first().unwrap();
                let source = &e.pattern.source.first().unwrap();
                event::trigger(auth, &e.bus, detail_type, source, &payload).await;
            } else {
                println!("Event not found")
            }
        }
        Entity::Route => {
            todo!()
        }

        Entity::Mutation => {
            todo!()
        }

        Entity::State => {
            todo!()
        },
        _ => todo!()
    }
}

pub async fn invoke(
    auth: &Auth,
    maybe_entity: Option<String>,
    topology: &Topology,
    payload: Option<String>,
    dumb: bool,
) {
    let dir = u::pwd();
    let payload = read_payload(auth, &dir, payload).await;

    let Topology { flow, fqn, namespace, sandbox, .. } = topology;

    match maybe_entity {
        Some(e) => {
            let (entity, component) = Entity::as_entity_component(&e);
            match component {
                Some(c) => invoke_component(auth, topology, entity, &c, &payload).await,
                None => repl::start(&auth.name, &namespace, &sandbox).await.expect("REASON")
            }
        },
        None => {
            let mode = match flow {
                Some(f) => &f.mode,
                None => "Standard",
            };

            state::invoke(auth, fqn, &payload, &mode, dumb).await
        }
    }
}

pub async fn run_local(payload: Option<String>) {
    let payload = read_payload_local(payload);
    function::invoke_local(&payload).await;
}
