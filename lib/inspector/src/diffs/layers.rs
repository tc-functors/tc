use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use aws::Env;
use configurator::Config;
use aws::layer;
use crate::cache;
use crate::cache::Layer;


fn build(xs: Vec<String>) -> Vec<Layer> {
    let mut v: Vec<Layer> = vec![];
    for x in xs {
        let m = Layer {
            name: x,
            dev: 0,
            stable: 0
        };
        v.push(m);
    }
    v
}

#[derive(Template)]
#[template(path = "diffs/layers_list.html")]
struct LayersTemplate {
    items: Vec<Layer>
}

pub async fn generate() -> impl IntoResponse {
    let layers = cache::find_resolved_layers().await;
    let xs = if layers.is_empty() {
        build(cache::find_layers().await)
    } else {
        layers
    };
    let t = LayersTemplate {
        items: xs
    };
    Html(t.render().unwrap())
}


pub async fn sync() -> impl IntoResponse {

    let layers = cache::find_layers().await;

    let cfg = Config::new(None, "");
    let profile = cfg.aws.lambda.layers_profile.unwrap();
    let env = Env::new(&profile, None, Config::new(None, &profile));
    let client = layer::make_client(&env).await;

    let mut resolve_layers: Vec<Layer> = vec![];

    for layer in layers {
        let dev = format!("{}-dev", &layer);
        let dev_version = layer::find_latest_version(&client, &dev).await;
        let stable_version = layer::find_latest_version(&client, &layer).await;
        tracing::debug!("dev - {}", &dev_version);
        let xl = Layer {
            name: layer,
            dev: dev_version,
            stable: stable_version
        };
        resolve_layers.push(xl);
    }
    cache::save_resolved_layers(resolve_layers.clone()).await;

    let t = LayersTemplate {
        items: resolve_layers
    };
    Html(t.render().unwrap())
}


#[derive(Template)]
#[template(path = "diffs/layers.html")]
struct ViewTemplate {
    entity: String,
    context: String,
    left: String,
    right: String
}

pub async fn view() -> impl IntoResponse {
    let temp = ViewTemplate {
        entity: String::from("layers"),
        context: String::from("diffs"),
        left: String::from("a"),
        right: String::from("b")
    };
    Html(temp.render().unwrap())
}
