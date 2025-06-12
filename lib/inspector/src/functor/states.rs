use crate::cache;
use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
};
use compiler::TopologySpec;

#[derive(Template)]
#[template(path = "functor/states.html")]
struct ListTemplate {
    root: String,
    namespace: String,
    definition: String,
}

fn lookup_spec(dir: &str) -> TopologySpec {
    let f = format!("{}/topology.yml", dir);
    TopologySpec::new(&f)
}

pub async fn list(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
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
    let temp = ListTemplate {
        root: root,
        namespace: namespace,
        definition: definition,
    };
    Html(temp.render().unwrap())
}
