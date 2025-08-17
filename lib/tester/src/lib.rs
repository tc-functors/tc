use authorizer::Auth;
use composer::{Function};
use composer::spec::TestSpec;
use invoker::aws::lambda;
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
        "matches" | "=" => assert_json_eq!(response, expected),
        "includes" | "contains" => assert_json_include!(actual: response, expected: expected),
        _ => assert_json_path(response, expected, &cond)
    }
}

async fn run_unit(auth: &Auth, name: &str, fname: &str, fqn: &str, t: &TestSpec) {
    let TestSpec { payload, expect, condition, .. } = t;
    let start = Instant::now();

    let dir = u::pwd();
    let client = lambda::make_client(&auth).await;
    let payload = invoker::read_payload(auth, &dir, payload.clone()).await;
    let maybe_response = lambda::invoke_sync(&client, fqn, &payload).await;
    let response = match maybe_response {
        Ok(r) => r,
        Err(_) => panic!("Failed to execute test")
    };
    assert_case(expect.clone(), &response, condition.clone());
    let duration = start.elapsed();
    let _ = println!("Test unit {} (function/{}) ({}) {:#}",
                     name, fname, "pass".green(), u::time_format(duration));
}

pub async fn test(auth: &Auth, sandbox: &str, function: &Function, unit: Option<String>) {
    let tspecs = &function.test;
    if let Some(u) = unit {
        if let Some(t) =  tspecs.get(&u) {
            let fqn = render(&function.fqn, sandbox);
            run_unit(auth, &u, &function.name, &fqn, &t).await;
        }
    } else {
        println!("Running all {} test units", &tspecs.len());
        for (name, tspec) in tspecs {
            let fqn = render(&function.fqn, sandbox);
            run_unit(auth, &name, &function.name, &fqn, &tspec).await;
        }
    }
}

pub fn interactive() {

}

pub async fn test_functions(
    auth: &Auth,
    functions: &HashMap<String, Function>,
    unit: Option<String>
) {

}
