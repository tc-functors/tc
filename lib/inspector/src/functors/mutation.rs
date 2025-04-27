use crate::cache;
use askama::Template;
use axum::{
    extract::Path,
    response::{
        Html,
        IntoResponse,
    },
};

#[derive(Template)]
#[template(path = "functors/mutation/gql.html")]
struct ViewTemplate {
    definition: String,
}

pub async fn compile(Path((root, namespace)): Path<(String, String)>) -> impl IntoResponse {
    let topology = cache::find_topology(&root, &namespace).await;
    let definition = if let Some(t) = topology {
        let maybe_mut = &t.mutations.get("default");
        match maybe_mut {
            Some(m) => m
                .types
                .values()
                .cloned()
                .collect::<Vec<String>>()
                .join("\n"),
            None => String::from("nonce"),
        }
    } else {
        String::from("default")
    };

    let temp = ViewTemplate {
        definition: definition,
    };
    Html(temp.render().unwrap())
}
