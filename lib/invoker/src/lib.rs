pub mod aws;
mod event;
mod function;
mod state;
pub mod route;
mod repl;
use authorizer::Auth;
use composer::{Entity, Topology, ConfigSpec};
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


fn find_bucket() -> String {
    let cfg = ConfigSpec::new(None);
    let maybe_bucket = cfg.tester.bucket;
    match maybe_bucket {
        Some(b) => b,
        None => match std::env::var("TC_TEST_BUCKET") {
            Ok(p) => p,
            Err(_) => panic!("No test bucket specified")
        }
    }
}
pub async fn read_payload(auth: &Auth, dir: &str, s: Option<String>) -> String {
    match s {
        Some(p) => {
            if p.starts_with("s3://")  {
                read_uri(auth, &p).await
            } else if p.starts_with("//") {
                let bucket = find_bucket();
                let key = u::second(&p, "//");
                let rp = format!("s3://{}/{}", &bucket, &key);
                //println!("reading bucket {}",  rp);
                read_uri(auth, &rp).await
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
                event::trigger(auth, &e, &payload).await;
            } else {
                println!("Event not found")
            }
        }
        Entity::Route => {
            let routes = &topology.routes;
            if let Some(r) = routes.get(component) {
                let res = route::request(auth, &r).await;
                println!("{:?}", res);
            }
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

pub async fn invoke_emulator(
    auth: &Auth,
    maybe_entity: Option<String>,
    topology: &Topology,
    payload: Option<String>,
) {
    let payload = read_payload_local(payload);
    match maybe_entity {
        Some(e) => {
            let (entity, component) = Entity::as_entity_component(&e);
            match entity {
                Entity::Function => function::invoke_emulator(&payload).await,
                Entity::State => {
                    if let Some(flow) = &topology.flow {
                        let dir = u::pwd();
                        let def = serde_json::to_string(&flow.definition).unwrap();
                        state::invoke_emulator(auth, &dir, &def, &topology.fqn, &payload).await;

                    }
                }
                _ => ()
            }
        },
        None => function::invoke_emulator(&payload).await
    }
}
