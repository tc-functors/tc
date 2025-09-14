use assert_json_diff::{
    assert_json_eq,
    assert_json_include,
};
use provider::Auth;
use colored::Colorize;
use composer::{
    Entity,
    Function,
    Topology,
    spec::TestSpec,
};
use provider::aws::{
    eventbridge,
    lambda,
    sfn,
};
use jsonpath_rust::JsonPath;
use kit as u;
use serde_json::Value;
use std::{
    collections::HashMap,
    time::Instant,
};

fn render(s: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    u::stencil(s, table)
}

fn assert_json_path(data: Value, expected: Value, path: &str) {
    let vec: Vec<&Value> = data.query(path).unwrap();
    assert_json_eq!(vec.iter().collect::<Vec<_>>(), vec![&expected]);
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
        _ => assert_json_path(response, expected, &cond),
    }
}

async fn invoke_function(auth: &Auth, fqn: &str, payload: &str) -> String {
    let client = lambda::make_client(&auth).await;
    let maybe_response = lambda::invoke_sync(&client, fqn, &payload).await;
    match maybe_response {
        Ok(r) => r,
        Err(_) => panic!("Failed to execute test"),
    }
}

fn get_fqn(namespace: &str, topology: &Topology, name: &str) -> Option<String> {
    if topology.namespace == namespace {
        if let Some(f) = &topology.functions.get(name) {
            Some(f.fqn.clone())
        } else {
            None
        }
    } else {
        if let Some(node) = topology.nodes.get(namespace) {
            if let Some(f) = node.functions.get(name) {
                Some(f.fqn.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

async fn invoke(
    auth: &Auth,
    namespace: &str,
    topology: &Topology,
    entity: &str,
    payload: &str,
) -> String {
    let (entity, component) = Entity::as_entity_component(entity);
    match entity {
        Entity::Function => {
            if let Some(c) = component {
                let maybe_fqn = get_fqn(namespace, topology, &c);
                match maybe_fqn {
                    Some(fqn) => invoke_function(auth, &fqn, payload).await,
                    None => panic!("No function defined"),
                }
            } else {
                panic!("Component not specified")
            }
        }
        Entity::State => {
            let client = sfn::make_client(auth).await;
            let arn = auth.sfn_arn(&topology.fqn);

            let mode = match &topology.flow {
                Some(f) => &f.mode,
                None => "Standard",
            };

            if mode == "Express" {
                let (exec_arn, _maybe_response) = match sfn::start_sync_execution(client, &arn, &payload, None).await {
                   Ok((exec_arn, maybe_response)) => (exec_arn, maybe_response),
                   Err(error) => panic!("Failed to invoke. Error: {}", error)
                };
                return exec_arn
            } else {
                let (exec_arn, _maybe_response) = match sfn::start_execution(client, &arn, &payload).await {
                    Ok((exec_arn, maybe_response)) => (exec_arn, maybe_response),
                    Err(error) => panic!("Failed to invoke. Error: {}", error)
                };
                return exec_arn
            }
        }
        Entity::Event => {
            if let Some(c) = component {
                if let Some(e) = &topology.events.get(&c) {
                    let client = eventbridge::make_client(auth).await;
                    let detail_type = &e.pattern.detail_type.first().unwrap();
                    let source = &e.pattern.source.first().unwrap();
                    eventbridge::put_event(client, &e.bus, detail_type, source, payload).await
                } else {
                    panic!("Event not found")
                }
            } else {
                panic!("No component defined")
            }
        }
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
        }

        _ => todo!(),
    }
}

async fn test_function_unit(auth: &Auth, fname: &str, fqn: &str, t: &TestSpec) {
    let TestSpec {
        name,
        payload,
        expect,
        condition,
        ..
    } = t;
    let start = Instant::now();
    let dir = u::pwd();
    let payload = invoker::read_payload(auth, &dir, payload.clone()).await;
    let response = invoke_function(auth, fqn, &payload).await;
    assert_case(expect.clone(), &response, condition.clone());
    let name = match name {
        Some(n) => n,
        None => fname,
    };
    let duration = start.elapsed();
    let _ = println!(
        "Test unit {} (function/{}) ({}) {:#}",
        name,
        fname,
        "pass".green(),
        u::time_format(duration)
    );
}

pub async fn test_function(auth: &Auth, sandbox: &str, function: &Function, unit: Option<String>) {
    let tspecs = &function.test;
    if let Some(u) = unit {
        if let Some(t) = tspecs.get(&u) {
            let fqn = render(&function.fqn, sandbox);
            test_function_unit(auth, &function.name, &fqn, &t).await;
        }
    } else {
        println!("Running all {} test units", &tspecs.len());
        for (_, tspec) in tspecs {
            let fqn = render(&function.fqn, sandbox);
            test_function_unit(auth, &function.name, &fqn, &tspec).await;
        }
    }
}

pub async fn test_topology_unit(
    auth: &Auth,
    namespace: &str,
    name: &str,
    topology: &Topology,
    spec: &TestSpec,
) {
    let dir = u::pwd();
    let TestSpec {
        payload,
        expect,
        condition,
        entity,
        ..
    } = spec;

    tracing::debug!("Testing {}/{}", namespace, name);

    let payload = invoker::read_payload(auth, &dir, payload.clone()).await;

    let start = Instant::now();
    let entity = u::maybe_string(entity.clone(), "state");

    let response = invoke(auth, namespace, topology, &entity, &payload).await;
    assert_case(expect.clone(), &response, condition.clone());

    let duration = start.elapsed();
    let _ = println!(
        "[{}] {}/{}/{} ({:#})",
        "pass".green(),
        namespace.cyan(),
        &entity,
        name,
        u::time_format(duration)
    );
}

fn get_tspecs(topology: &Topology) -> HashMap<String, TestSpec> {
    let mut tests: HashMap<String, TestSpec> = HashMap::new();
    for (name, mut spec) in topology.tests.clone() {
        spec.name = Some(name);
        spec.namespace = Some(topology.namespace.clone());
        tests.insert(u::uuid_str(), spec);
    }

    for (_, node) in &topology.nodes {
        for (name, mut spec) in node.tests.clone() {
            spec.name = Some(name);
            spec.namespace = Some(node.namespace.clone());
            tests.insert(u::uuid_str(), spec);
        }
    }
    tests
}

async fn test_topology_aux(auth: &Auth, name: String, spec: TestSpec, topology: &Topology) {
    let cname = u::maybe_string(spec.name.clone(), &name);
    let namespace = u::maybe_string(spec.namespace.clone(), &topology.namespace);
    test_topology_unit(auth, &namespace, &cname, topology, &spec).await;
}

pub async fn test_topology(auth: &Auth, topology: &Topology, unit: Option<String>) {
    let tspecs = get_tspecs(topology);

    if let Some(u) = unit {
        if let Some(spec) = topology.tests.get(&u) {
            test_topology_unit(auth, &topology.namespace, &u, topology, spec).await;
        }
    } else {
        let mut tasks = vec![];
        for (name, spec) in tspecs {
            let a = auth.clone();
            let t = topology.clone();
            let h = tokio::spawn(async move {
                test_topology_aux(&a, name, spec, &t).await;
            });
            tasks.push(h);
        }
        for task in tasks {
            let _ = task.await;
        }
    }
}
