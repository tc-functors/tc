use askama::Template;
use axum::{
    response::{Html, IntoResponse},
};

use serde_derive::{Deserialize, Serialize};

use aws::Env;
use configurator::Config;
use aws::layer;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Layer {
    name: String,
    version: String,
    created: String
}

fn build(xs: Vec<layer::Layer>) -> Vec<Layer> {
    let mut v: Vec<Layer> = vec![];
    for x in xs {
        let m = Layer {
            name: x.name,
            version: x.version.to_string(),
            created: x.created_date
        };
        v.push(m);
    }
    v
}

async fn read_cache() -> Vec<Layer> {
    let key = "layers";
    if cache::has_key(key) {
        tracing::info!("Found cache: {}", key);
        let s = cache::read(key);
        let r: Vec<Layer> = serde_json::from_str(&s).unwrap();
        r
    } else {
        vec![]
    }
}


#[derive(Template)]
#[template(path = "builds/fragments/layers.html")]
struct LayersTemplate {
    items: Vec<Layer>
}



pub async fn list() -> impl IntoResponse {
    let layers = read_cache().await;
    let t = LayersTemplate {
        items: layers
    };
    Html(t.render().unwrap())
}


pub async fn sync() -> impl IntoResponse {
    let cfg = Config::new(None, "");
    let profile = cfg.aws.lambda.layers_profile.unwrap();
    let env = Env::new(&profile, None, Config::new(None, &profile));
    let client = layer::make_client(&env).await;
    let xs = layer::list_all_layers(&client).await;
    let layers = build(xs);
    cache::write("layers", &serde_json::to_string(&layers).unwrap()).await;
    let t = LayersTemplate {
        items: layers
        };
    Html(t.render().unwrap())
}
