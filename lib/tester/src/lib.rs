use authorizer::Auth;
use composer::{Function};
use composer::spec::TestSpec;
use invoker::aws::lambda;
use std::collections::HashMap;
use kit as u;
use serde_json::Value;
use jsonpath_rust::query::QueryRef;
use jsonpath_rust::JsonPath;
use assert_json_diff::{assert_json_eq, assert_json_include};

fn render(s: &str, sandbox: &str) -> String {
    let mut table: HashMap<&str, &str> = HashMap::new();
    table.insert("sandbox", sandbox);
    u::stencil(s, table)
}

fn query_path(data: Value, path: &str) -> String {
    println!("query path");
    let vec: Vec<QueryRef<Value>> = data.query_with_path(path).unwrap();
    println!("-> {:?}", &vec);
}

fn assert_case(expected: Option<String>, response: &str, cond: Option<String>) {
    let expected = u::maybe_string(expected, "{}");
    let expected = u::json_value(&expected);
    let response = u::json_value(&response);
    let cond = u::maybe_string(cond, "matches");
    match cond.as_ref() {
        "matches" => assert_json_eq!(response, expected),
        "includes" | "contains" => assert_json_include!(actual: response, expected: expected),
        _ => query_path(response, &cond)
    }
}

async fn run_unit(auth: &Auth, fqn: &str, t: &TestSpec) {
    let TestSpec { payload, expect, condition, .. } = t;
    let dir = u::pwd();
    let client = lambda::make_client(&auth).await;
    let payload = invoker::read_payload(auth, &dir, payload.clone()).await;
    let maybe_response = lambda::invoke_sync(&client, fqn, &payload).await;
    let response = match maybe_response {
        Ok(r) => r,
        Err(_) => panic!("Failed to execute test")
    };
    assert_case(expect.clone(), &response, condition.clone());
    println!("Test Passed");
}

pub async fn test(auth: &Auth, sandbox: &str, function: &Function, unit: Option<String>) {
    let tspecs = &function.test;
    if let Some(u) = unit {
        if let Some(t) =  tspecs.get(&u) {
            println!("Testing unit {}", &u);
            let fqn = render(&function.fqn, sandbox);
            run_unit(auth, &fqn, &t).await;
        }
    } else {
        println!("Running all test units {}", &tspecs.len());
        for (name, tspec) in tspecs {
            println!("Testing unit {}", &name);
            let fqn = render(&function.fqn, sandbox);
            run_unit(auth, &fqn, &tspec).await;
        }
    }
}

pub async fn test_functions(
    auth: &Auth,
    functions: &HashMap<String, Function>,
    unit: Option<String>
) {

}
