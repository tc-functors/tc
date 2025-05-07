mod image;
mod layer;
mod aws;

use compiler::spec::{
    BuildKind,
    BuildOutput,
};
use authorizer::Auth;
use std::collections::HashMap;

pub async fn publish(auth: &Auth, build: BuildOutput) {
    let BuildOutput {
        dir,
        kind,
        artifact,
        runtime,
        name,
        ..
    } = build;

    let lang = runtime.to_str();
    match kind {
        BuildKind::Layer | BuildKind::Library => {
            if layer::should_split(&dir) {
                layer::publish(auth, &lang, &format!("{}-0-dev", &name), "deps1.zip").await;
                layer::publish(auth, &lang, &format!("{}-1-dev", &name), "deps2.zip").await;
            } else {
                let layer_name = format!("{}-dev", &name);
                layer::publish(auth, &lang, &layer_name, &artifact).await;
            }
        }
        BuildKind::Image => image::publish(auth, &artifact).await,
        _ => (),
    }
}

pub async fn publish_as_dev(auth: &Auth, layer_name: &str, lang: &str) {
    layer::publish_as_dev(auth, layer_name, lang).await
}

pub async fn promote(auth: &Auth, layer_name: &str, lang: &str, version: Option<String>) {
    layer::promote(auth, layer_name, lang, version).await;
}

pub async fn demote(auth: &Auth, name: Option<String>, lang: &str) {
    match name {
        Some(p) => {
            publish_as_dev(auth, &p, lang).await;
        }
        None => {
            let layers = compiler::find_layers();
            let mut h: HashMap<String, String> = HashMap::new();
            for layer in layers {
                h.insert(layer.name.to_owned(), layer.runtime.to_str());
            }
            for (name, lang) in h {
                publish_as_dev(auth, &name, &lang).await
            }
        }
    }
}

pub async fn download_layer(auth: &Auth, name: &str) {
    layer::download(auth, name).await
}
