use crate::Store;
use askama::Template;
use axum::{
    Form,
    extract::{
        Path,
        State,
    },
    response::{
        Html,
        IntoResponse,
    },
};
use composer::{Function, Policy};
use compiler::Entity;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Template)]
#[template(path = "functor/function/build.html")]
struct FlowTemplate {}

pub async fn build(Path((_root, _namespace)): Path<(String, String)>) -> impl IntoResponse {
    let temp = FlowTemplate {};
    Html(temp.render().unwrap())
}

#[derive(Template)]
#[template(path = "functor/function/compose.html")]
struct DataTemplate {
    definition: String,
}

#[derive(Deserialize, Debug)]
pub struct FunctionInput {
    pub function: String,
}

pub async fn compose(
    State(store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
    Form(payload): Form<FunctionInput>,
) -> impl IntoResponse {
    let FunctionInput { function } = payload;
    let function = store.find_function(&root, &namespace, &function).await;

    let definition = if let Some(f) = function {
        serde_json::to_string_pretty(&f).unwrap()
    } else {
        String::from("")
    };
    let temp = DataTemplate {
        definition: definition,
    };
    Html(temp.render().unwrap())
}

pub async fn permissions(
    State(store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
    Form(payload): Form<FunctionInput>,
) -> impl IntoResponse {
    let FunctionInput { function } = payload;
    let function = store.find_function(&root, &namespace, &function).await;

    let definition = if let Some(f) = function {
        f.runtime.role.policy
    } else {
        Policy::new(Entity::Function)
    };
    let definition = serde_json::to_string(&definition).unwrap();
    let temp = DataTemplate {
        definition: definition,
    };
    Html(temp.render().unwrap())
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
struct ListTemplate {
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

pub async fn list(
    State(store): State<Store>,
    Path((root, namespace)): Path<(String, String)>,
) -> impl IntoResponse {
    let fns = store.find_functions(&root, &namespace).await;
    let temp = ListTemplate {
        root: root,
        namespace: namespace,
        items: build_functions(fns),
    };
    Html(temp.render().unwrap())
}
