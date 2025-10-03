pub mod runtime;

use super::template;
use compiler::{
    Entity,
    spec::{
        BuildKind,
        BuildSpec,
        TestSpec,
        function::FunctionSpec,
    },
};

pub use runtime::Runtime;
use kit::*;
use serde_derive::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use safe_unwrap::safe_unwrap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Target {
    pub entity: Entity,
    pub name: String,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Build {
    pub dir: String,
    pub kind: BuildKind,
    pub pre: Vec<String>,
    pub post: Vec<String>,
    pub version: Option<String>,
    pub command: String,
    pub shared_context: bool,
    pub skip_dev_deps: bool,
    pub environment: HashMap<String, String>,
}

impl Build {
    pub fn new(
        dir: &str,
        bspec: Option<BuildSpec>,
        tasks: HashMap<String, String>,
    ) -> Build {
        match bspec {
            Some(b) => Build {
                dir: s!(dir),
                kind: b.kind,
                pre: b.pre,
                post: b.post,
                command: b.command,
                version: b.version,
                shared_context: match b.shared_context {
                    Some(s) => s,
                    None => true,
                },
                skip_dev_deps: match b.skip_dev_deps {
                    Some(s) => s,
                    None => true,
                },
                environment: HashMap::new(),
            },
            None => {
                let command = match tasks.get("build") {
                    Some(c) => c.to_owned(),
                    None => s!("zip -9 -q lambda.zip *.*"),
                };

                Build {
                    dir: s!(dir),
                    kind: BuildKind::Code,
                    pre: vec![],
                    post: vec![],
                    version: None,
                    command: command,
                    shared_context: false,
                    skip_dev_deps: false,
                    environment: HashMap::new(),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Function {
    pub name: String,
    pub actual_name: String,
    pub namespace: String,
    pub dir: String,
    pub description: Option<String>,
    pub fqn: String,
    pub arn: String,
    pub layer_name: Option<String>,
    pub version: String,
    pub runtime: Runtime,
    pub build: Build,
    pub test: HashMap<String, TestSpec>,
    pub targets: Vec<Target>,
}

fn make_test(t: Option<HashMap<String, TestSpec>>) -> HashMap<String, TestSpec> {
    match t {
        Some(spec) => spec,
        None => HashMap::new(),
    }
}

impl Function {
    pub fn new(dir: &str, namespace: &str, name: &str, spec: &FunctionSpec) -> Function {

        let fqn = safe_unwrap!("No fqn defined", spec.fqn.clone());
        let runtime = Runtime::new(spec);

        let build = Build::new(dir, spec.build.clone(), spec.tasks.clone());

        Function {
            dir: dir.to_string(),
            name: name.to_string(),
            actual_name: name.to_string(),
            arn: template::lambda_arn(&fqn),
            version: s!(""),
            fqn: fqn,
            description: None,
            namespace: namespace.to_string(),
            build: build,
            layer_name: spec.layer_name.clone(),
            test: make_test(spec.test.clone()),
            runtime: runtime,
            targets: vec![],
        }
    }


}
