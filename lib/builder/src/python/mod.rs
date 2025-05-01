mod code;
mod extension;
mod image;
mod inline;
mod layer;
mod library;
mod slab;

use super::BuildOutput;
use compiler::{
    Build,
    Runtime,
    spec::{
        BuildKind,
        LangRuntime,
    },
};
use kit::sh;

pub fn build(
    dir: &str,
    lang: &LangRuntime,
    runtime: &Runtime,
    name: &str,
    spec: Build,
    image_kind: &str,
) -> BuildOutput {
    let Build {
        kind,
        pre,
        post,
        command,
        ..
    } = spec;
    let Runtime { uri, .. } = runtime;

    let path = match kind {
        BuildKind::Code => code::build(dir, &command),
        BuildKind::Inline => inline::build(dir, name, &lang, &command),
        BuildKind::Layer => layer::build(dir, name, &lang, pre, post),
        BuildKind::Library => library::build(dir, name),
        BuildKind::Slab => slab::build(dir, name, &lang, pre, post),
        BuildKind::Extension => extension::build(dir, name),
        BuildKind::Image => image::build(dir, name, &lang, image_kind, &spec.images, uri),
        BuildKind::Runtime => todo!(),
    };
    BuildOutput {
        name: String::from(name),
        dir: dir.to_string(),
        artifact: path,
        kind: kind,
        runtime: lang.clone(),
    }
}

pub fn clean(dir: &str) {
    sh("rm -rf lambda.zip dist __pycache__", dir);
}
