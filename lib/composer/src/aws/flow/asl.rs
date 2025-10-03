#![allow(non_snake_case)]
use super::template;
use compiler::spec::function::FunctionSpec;
use kit as u;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Step {
    Type: String,
    Resource: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    Next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    End: Option<bool>,
}

impl Step {
    fn new(name: &str, next: &str) -> Step {
        let fn_name = format!("{{{{namespace}}}}_{name}_{{{{sandbox}}}}");
        if next == "end" {
            Step {
                Type: s!("Task"),
                Resource: template::lambda_arn(&fn_name),
                Next: None,
                End: Some(true),
            }
        } else {
            Step {
                Type: s!("Task"),
                Resource: template::lambda_arn(&fn_name),
                Next: Some(s!(next)),
                End: None,
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct StepFunction {
    Comment: String,
    StartAt: String,
    TimeoutSeconds: u16,
    States: HashMap<String, Step>,
}

impl StepFunction {
    fn new(root: &str, graph: HashMap<String, String>) -> StepFunction {
        let mut steps: HashMap<String, Step> = HashMap::new();
        for (name, next) in graph {
            let step = Step::new(&name, &next);
            steps.insert(name, step);
        }

        StepFunction {
            Comment: s!(""),
            StartAt: s!(root),
            TimeoutSeconds: 600,
            States: steps,
        }
    }
}

fn as_bool(b: Option<bool>) -> bool {
    match b {
        Some(p) => p,
        None => false,
    }
}

pub fn generate(fns: HashMap<String, FunctionSpec>) -> Value {
    let mut root = String::from("");
    let mut graph: HashMap<String, String> = HashMap::new();
    for (name, f) in fns {
        if as_bool(f.root) {
            root = name.clone();
        }
        let next = match f.function {
            Some(child) => child,
            None => s!("end"),
        };
        graph.insert(name, next);
    }
    let def = StepFunction::new(&root, graph);
    let data = serde_json::to_string(&def).unwrap();
    u::json_value(&data)
}
