use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    Form,
};

use compiler::{TopologyCount, Topology};

use serde::Deserialize;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Functor {
    id: String,
    namespace: String,
    env: String,
    sandbox: String,
    functions: usize,
    nodes: usize,
    events: usize,
    queues: usize,
    routes: usize,
    mutations: usize,
    states: usize,
    version: String
}

async fn find_functors() -> Vec<Functor> {
    let items = cache::list();
    let mut xs: Vec<Functor> = vec![];
    for item in items {
        let key = cache::make_key(&item.namespace, &item.env, &item.sandbox);
        let maybe_topology = cache::read_topology(&key).await;
        if let Some(topology) = maybe_topology {
            let t = TopologyCount::new(&topology);
            let f = Functor {
                id: key,
                namespace: item.namespace,
                env: item.env,
                sandbox: item.sandbox,
                functions: t.functions,
                nodes: t.nodes,
                events: t.events,
                queues: t.queues,
                routes: t.routes,
                states: t.states,
                mutations: t.mutations,
                version: String::from(&topology.version)
            };
            xs.push(f)
        }
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs.reverse();
    xs
}


#[derive(Template)]
#[template(path = "functors_list.html")]
struct FunctorsTemplate {
    items: Vec<Functor>
 }

#[derive(Deserialize, Debug)]
pub struct FunctorsInput {
    pub env: String,
    pub sandbox: String,
}

pub async fn search_functors(Form(payload): Form<FunctorsInput>) -> impl IntoResponse {
    let FunctorsInput { env, sandbox, .. } = payload;

    let xs = find_functors().await;
    tracing::debug!("search {} - {}", env, sandbox);

    let functors = xs.into_iter().filter(|x| &x.env == &env && &x.sandbox == &sandbox).collect();

    let t = FunctorsTemplate {
        items: functors
    };
    Html(t.render().unwrap())
}


pub async fn list_functors() -> impl IntoResponse {
    let xs = find_functors().await;
    let functors: Vec<_> = xs.into_iter().filter(|x| x.version != "0.0.1").collect();
    let t = FunctorsTemplate {
        items: functors
    };
    Html(t.render().unwrap())
}

struct Manifest {
    namespace: String,
    prod: String,
    qa: String,
    staging: String,
    poc: String
}


#[derive(Template)]
#[template(path = "manifests_list.html")]
struct ManifestsTemplate {
    items: Vec<Manifest>
 }

pub async fn list_manifests() -> impl IntoResponse {
    let manifests = vec![];
    let t = ManifestsTemplate {
        items: manifests
    };
    Html(t.render().unwrap())
}

pub async fn search_manifests() -> impl IntoResponse {
    let manifests = vec![];
    let t = ManifestsTemplate {
        items: manifests
    };
    Html(t.render().unwrap())
}


// nodes

#[derive(Template)]
#[template(path = "nodes_list.html")]
struct NodesTemplate {
    id: String,
    items: Vec<Functor>
 }

async fn as_functors(root: &Topology) -> Vec<Functor> {
    let mut xs: Vec<Functor> = vec![];
    tracing::debug!("{}", &root.nodes.len());
    for node in &root.nodes {
        let t = TopologyCount::new(&node);
        let f = Functor {
            id: node.fqn.clone(),
            namespace: node.namespace.clone(),
            env: node.env.clone(),
            sandbox: node.sandbox.clone(),
            functions: t.functions,
            nodes: t.nodes,
            events: t.events,
            queues: t.queues,
            routes: t.routes,
            states: t.states,
            mutations: t.mutations,
            version: String::from(&node.version)
        };
        xs.push(f)
    }
    xs.sort_by(|a, b| b.namespace.cmp(&a.namespace));
    xs.reverse();
    xs
}

pub async fn get_nodes(Path(id): Path<String>) -> impl IntoResponse {
    let maybe_topology = cache::read_topology(&id).await;

    if let Some(t) = maybe_topology {
        tracing::debug!("Found topology");
        let temp = NodesTemplate {
            id: id,
            items: as_functors(&t).await
        };
        Html(temp.render().unwrap())
    } else {
        let temp = NodesTemplate {
            id: id,
            items: vec![]
        };
        Html(temp.render().unwrap())
    }
}

// functions

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Function {
    name: String,
    dir: String,
    fqn: String,
    layers: Vec<String>,
    memory: i32,
    timeout: i32,
    runtime: String,
}

#[derive(Template)]
#[template(path = "functions_list.html")]
struct FunctionsTemplate {
    items: Vec<Function>
 }


fn build_functions(t: &Topology) -> Vec<Function> {
    let mut xs: Vec<Function> = vec![];

    for (dir, f) in &t.functions {
        let fun = Function {
            name: f.actual_name.clone(),
            dir: dir.to_string(),
            fqn: f.fqn.clone(),
            layers: f.runtime.layers.clone(),
            memory: f.runtime.memory_size.unwrap(),
            timeout: f.runtime.timeout.unwrap(),
            runtime: f.runtime.lang.to_str()
        };
        xs.push(fun);
        for node in &t.nodes {
            for (d, nf) in &node.functions {
                let fun = Function {
                    name: nf.actual_name.clone(),
                    dir: d.to_string(),
                    fqn: nf.fqn.clone(),
                    layers: nf.runtime.layers.clone(),
                    memory: nf.runtime.memory_size.unwrap(),
                    timeout: nf.runtime.timeout.unwrap(),
                    runtime: nf.runtime.lang.to_str()
                };
                xs.push(fun);
            }
        }
    }
    xs.dedup_by(|a, b| a.name == b.name);
    xs.sort_by(|a, b| b.name.cmp(&a.name));
    xs
}

pub async fn get_functions(Path(id): Path<String>) -> impl IntoResponse {
    let maybe_topology = cache::read_topology(&id).await;

    if let Some(t) = maybe_topology {
        tracing::debug!("Found topology");
        let temp = FunctionsTemplate {
            items: build_functions(&t)
        };
        Html(temp.render().unwrap())
    } else {
        let temp = NodesTemplate {
            id: id,
            items: vec![]
        };
        Html(temp.render().unwrap())
    }
}
