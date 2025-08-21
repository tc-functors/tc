use authorizer::Auth;
use composer::{Function, Entity, Topology};
use composer::spec::TestSpec;
use invoker::aws::{lambda, sfn, eventbridge};
use std::collections::HashMap;
use colored::Colorize;
use kit as u;
use serde_json::Value;
use jsonpath_rust::JsonPath;
use assert_json_diff::{assert_json_eq, assert_json_include};
use std::time::Instant;

fn render(s: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    u::stencil(s, table)
}

fn assert_json_path(data: Value, expected: Value, path: &str) {
    let vec: Vec<&Value> = data.query(path).unwrap();
    assert_json_eq!(
        vec.iter().collect::<Vec<_>>(),
        vec![&expected]
    );
}

fn assert_case(expected: Option<String>, response: &str, cond: Option<String>) {
    let expected = u::maybe_string(expected, "{}");
    let expected = u::json_value(&expected);
    let response = u::json_value(&response);
    let cond = u::maybe_string(cond, "matches");
    match cond.as_ref() {
        "matches" | "=" => {
            assert_json_eq!(response, expected)
        }
        "includes" | "contains" => {
            assert_json_include!(actual: response, expected: expected)
        }
        _ => assert_json_path(response, expected, &cond)
    }
}

async fn invoke_function(auth: &Auth, fqn: &str, payload: &str) -> String {
    let client = lambda::make_client(&auth).await;
    let maybe_response = lambda::invoke_sync(&client, fqn, &payload).await;
    match maybe_response {
        Ok(r) => r,
        Err(_) => panic!("Failed to execute test")
    }
}

async fn invoke(auth: &Auth, topology: &Topology, entity: &str, payload: &str) -> String {
    let (entity, component) = Entity::as_entity_component(entity);
    match entity {
        Entity::Function => {
            if let Some(c) = component {
                if let Some(function) = &topology.functions.get(&c) {
                    invoke_function(auth, &function.fqn, payload).await
                } else {
                    panic!("No function defined")
                }
            } else {
                panic!("Component not specified")
            }
        },
        Entity::State => {
            let client = sfn::make_client(auth).await;
            let arn = auth.sfn_arn(&topology.fqn);

            let mode = match &topology.flow {
                Some(f) => &f.mode,
                None => "Standard",
            };
            let maybe_response = if mode == "Express" {
                sfn::start_sync_execution(client, &arn, &payload, None).await
            } else {
                let id = sfn::start_execution(client, &arn, &payload).await;
                Some(id)
            };
            match maybe_response {
                Some(r) => r,
                None => panic!("Failed to execute test")
            }
        },
        Entity::Event => {
            if let Some(c) = component {
                if let Some(e) = &topology.events.get(&c) {
                    let client = eventbridge::make_client(auth).await;
                    let detail_type = &e.pattern.detail_type.first().unwrap();
                    let source = &e.pattern.source.first().unwrap();
                    eventbridge::put_event(
                        client, &e.bus, detail_type, source, payload
                    ).await
                } else {
                    panic!("Event not found")
                }
            } else {
                panic!("No component defined")
            }
        },
        Entity::Route => {
            if let Some(c) = component {
                if let Some(r) = &topology.routes.get(&c) {
                    let res = invoker::route::request(auth, &r).await;
                    res.to_string()
                } else {
                    panic!("Route not found")
                }
            } else {
                panic!("No component defined")
            }

        },

        _ => todo!()

    }
}

async fn test_function_unit(auth: &Auth, name: &str, fname: &str, fqn: &str, t: &TestSpec) {
    let TestSpec { payload, expect, condition, .. } = t;
    let start = Instant::now();
    let dir = u::pwd();
    let payload = invoker::read_payload(auth, &dir, payload.clone()).await;
    let response = invoke_function(auth, fqn, &payload).await;
    assert_case(expect.clone(), &response, condition.clone());
    let duration = start.elapsed();
    let _ = println!("Test unit {} (function/{}) ({}) {:#}",
                     name, fname, "pass".green(), u::time_format(duration));
}

pub async fn test_function(auth: &Auth, sandbox: &str, function: &Function, unit: Option<String>) {
    let tspecs = &function.test;
    if let Some(u) = unit {
        if let Some(t) =  tspecs.get(&u) {
            let fqn = render(&function.fqn, sandbox);
            test_function_unit(auth, &u, &function.name, &fqn, &t).await;
        }
    } else {
        println!("Running all {} test units", &tspecs.len());
        for (name, tspec) in tspecs {
            let fqn = render(&function.fqn, sandbox);
            test_function_unit(auth, &name, &function.name, &fqn, &tspec).await;
        }
    }
}

pub async fn test_topology_unit(auth: &Auth, name: &str, topology: &Topology, spec: &TestSpec) {
    let dir = u::pwd();
    let TestSpec { payload, expect, condition, entity, .. } = spec;

    let payload = invoker::read_payload(auth, &dir, payload.clone()).await;

    let start = Instant::now();
    let entity = u::maybe_string(entity.clone(), "state");

    let response = invoke(auth, topology, &entity, &payload).await;
    assert_case(expect.clone(), &response, condition.clone());

    let duration = start.elapsed();
    let _ = println!("Test unit {} ({}) ({}) {:#}",
                     name, &entity, "pass".green(), u::time_format(duration));
}

pub async fn test_topology(
    auth: &Auth,
    topology: &Topology,
    unit: Option<String>
) {

    let tspecs = &topology.tests;

    if let Some(u) = unit {
        if let Some(spec) = tspecs.get(&u) {
            test_topology_unit(auth, &u, topology, spec).await;
        }
    } else {
        for (name, spec) in tspecs {
            test_topology_unit(auth, &name, topology, spec).await;
        }
    }
}
