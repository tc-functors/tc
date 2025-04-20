mod layer;
mod ecr;

use aws::Env;
use std::collections::HashMap;
use compiler::spec::{BuildKind, BuildOutput};

pub async fn list_layers(env: &Env, layer_names: Vec<String>) -> String {
    layer::list(env, layer_names).await
}

pub async fn publish(env: &Env, build: BuildOutput) {

    let BuildOutput { dir, kind, artifact, runtime, name, .. } = build;

    let lang = runtime.to_str();
    match kind {
        BuildKind::Layer | BuildKind::Library => {
            if layer::should_split(&dir) {
                layer::publish(env, &lang, &format!("{}-0-dev", &name),
                               "deps1.zip").await;
                layer::publish(env, &lang, &format!("{}-1-dev", &name),
                               "deps2.zip").await;
            } else {
                let layer_name = format!("{}-dev", &name);
                layer::publish(env, &lang, &layer_name, &artifact).await;
            }
        },
        BuildKind::Image => ecr::publish(env, &artifact).await,
        _ => ()
    }
}

pub async fn publish_as_dev(env: &Env, layer_name: &str, lang: &str) {
    layer::publish_as_dev(env, layer_name, lang).await
}

pub async fn promote(env: &Env, layer_name: &str, lang: &str, version: Option<String>) {
    layer::promote(env, layer_name, lang, version).await;
}

pub async fn demote(env: &Env, name: Option<String>, lang: &str) {
    match name {
        Some(p) => {
            publish_as_dev(env, &p, lang).await;
        }
        None => {
            let layers = compiler::find_layers();
            let mut h: HashMap<String, String> = HashMap::new();
            for layer in layers {
                h.insert(layer.name.to_owned(), layer.runtime.to_str());
            }
            for (name, lang) in h {
                publish_as_dev(env, &name, &lang).await
            }
        }
    }
}

pub async fn list(env: &Env, kind: &BuildKind) {
    match kind {
        BuildKind::Layer => {
            let layer_names = compiler::find_layer_names();
            let table = list_layers(env, layer_names).await;
            println!("{}", table);
        },
        BuildKind::Image => {
            let images = ecr::list(env, "xxx").await;
            println!("{:?}", images);
        },
        _ => todo!()
    }
}

pub async fn download_layer(env: &Env, name: &str) {
    layer::download(env, name).await
}
