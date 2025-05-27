use crate::cache;
use askama::Template;
use axum::{
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
};
use compiler::{
    Function,
    TopologySpec,
};
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "functor/topology.html")]
struct DefinitionTemplate {
    root: String,
    namespace: String,
    definition: String,
}

fn lookup_definition(dir: &str) -> String {
    let f = format!("{}/topology.yml", dir);
    kit::slurp(&f)
}

pub async fn definition(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let f = cache::find_topology(&root, &namespace).await;
    if let Some(t) = f {
        let definition = lookup_definition(&t.dir);
        let temp = DefinitionTemplate {
            root: root,
            namespace: namespace,
            definition: definition,
        };
        Html(temp.render().unwrap())
    } else {
        let temp = DefinitionTemplate {
            root: root,
            namespace: namespace,
            definition: String::from("test"),
        };
        Html(temp.render().unwrap())
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Item {
    name: String,
    package_type: String,
    runtime: String,
    build: String,
}

#[derive(Template)]
#[template(path = "functor/functions.html")]
struct FunctionsTemplate {
    root: String,
    namespace: String,
    items: Vec<Item>,
}

fn build_functions(fns: HashMap<String, Function>) -> Vec<Item> {
    let mut xs: Vec<Item> = vec![];
    for (name, f) in fns {
        let item = Item {
            name: name,
            package_type: f.runtime.package_type.clone(),
            runtime: f.runtime.lang.to_str(),
            build: f.build.kind.to_str(),
        };
        xs.push(item);
    }
    xs
}

pub async fn functions(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let fns = cache::find_functions(&root, &namespace).await;
    let temp = FunctionsTemplate {
        root: root,
        namespace: namespace,
        items: build_functions(fns),
    };
    Html(temp.render().unwrap())
}

// mutations

fn lookup_spec(dir: &str) -> TopologySpec {
    let f = format!("{}/topology.yml", dir);
    TopologySpec::new(&f)
}

#[derive(Template)]
#[template(path = "functor/mutations.html")]
struct MutationsTemplate {
    root: String,
    namespace: String,
    definition: String,
}

pub async fn mutations(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let topology = cache::find_topology(&root, &namespace).await;
    let definition = if let Some(t) = topology {
        let spec = lookup_spec(&t.dir);
        match spec.mutations {
            Some(m) => serde_yaml::to_string(&m).unwrap(),
            None => String::from(""),
        }
    } else {
        String::from("")
    };
    let temp = MutationsTemplate {
        root: root,
        namespace: namespace,
        definition: definition,
    };
    Html(temp.render().unwrap())
}

// flow

#[derive(Template)]
#[template(path = "functor/states.html")]
struct StatesTemplate {
    root: String,
    namespace: String,
    definition: String,
}

pub async fn states(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let topology = cache::find_topology(&root, &namespace).await;
    let definition = if let Some(t) = topology {
        let spec = lookup_spec(&t.dir);
        match spec.flow {
            Some(m) => serde_yaml::to_string(&m).unwrap(),
            None => String::from(""),
        }
    } else {
        String::from("")
    };
    let temp = StatesTemplate {
        root: root,
        namespace: namespace,
        definition: definition,
    };
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "index.html")]
struct PageTemplate {
    context: String,
    root: String,
    namespace: String,
}

pub async fn page(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = PageTemplate {
        context: String::from("functors"),
        root: root,
        namespace: namespace,
    };
    Html(temp.render().unwrap())
}
