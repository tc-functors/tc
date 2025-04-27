use crate::cache;
use askama::Template;
use axum::response::{
    Html,
    IntoResponse,
};
use std::collections::HashMap;

struct Change {
    name: String,
    changes: String,
    version: String,
}

async fn build_changelog() -> Vec<Change> {
    let topologies = cache::find_all_topologies().await;
    let mut xs: Vec<Change> = vec![];
    for (name, t) in topologies {
        let changes = releaser::changelogs_since_last(&name, &t.version);
        let change = Change {
            name: name,
            changes: changes,
            version: t.version.clone(),
        };
        xs.push(change);
    }
    xs
}

#[derive(Template)]
#[template(path = "releases/current_changelog.html")]
struct ChangelogTemplate {
    items: Vec<Change>,
}

pub async fn changelog() -> impl IntoResponse {
    let t = ChangelogTemplate {
        items: build_changelog().await,
    };
    Html(t.render().unwrap())
}

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
#[template(path = "releases/current_versions.html")]
struct VersionsTemplate {
    items: HashMap<String, String>,
}

pub async fn versions() -> impl IntoResponse {
    let t = VersionsTemplate {
        items: build_versions().await,
    };
    Html(t.render().unwrap())
}

#[derive(Template)]
#[template(path = "releases/current.html")]
struct ViewTemplate {
    entity: String,
    context: String,
}

pub async fn view() -> impl IntoResponse {
    let t = ViewTemplate {
        entity: String::from("current"),
        context: String::from("releases"),
    };
    Html(t.render().unwrap())
}
