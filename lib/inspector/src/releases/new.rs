use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};
use std::collections::HashMap;
use crate::cache;

async fn build_versions() -> HashMap<String, String> {
    let topologies = cache::find_all_topologies().await;
    let mut h: HashMap<String, String> = HashMap::new();
    for (name, t) in topologies {
        //FIXME: get version from env
        h.insert(name, t.version);
    }
    h
}

#[derive(Template)]
#[template(path = "releases/new_versions.html")]
struct VersionsTemplate {
    items: HashMap<String, String>
}

pub async fn versions() -> impl IntoResponse {
    let t = VersionsTemplate {
        items: build_versions().await
    };
    Html(t.render().unwrap())
}


#[derive(Template)]
#[template(path = "releases/new.html")]
struct ViewTemplate {
    entity: String,
    context: String,
}

pub async fn view() -> impl IntoResponse {
    let t = ViewTemplate {
        entity: String::from("new"),
        context: String::from("releases"),
    };
    Html(t.render().unwrap())
}
