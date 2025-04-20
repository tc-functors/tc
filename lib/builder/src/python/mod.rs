mod layer;
mod image;
mod inline;
mod extension;
mod code;
mod library;
mod slab;

use super::BuildOutput;
use compiler::spec::{BuildKind, LangRuntime};
use compiler::{Build, Runtime};
use kit::sh;
use kit as u;

pub fn build(
    dir: &str,
    runtime: &Runtime,
    name: &str,
    spec: Build,
    image_kind: &str
) -> BuildOutput {


    let Build { kind, pre, post, command, .. } = spec;
    let Runtime { lang, uri, .. } = runtime;

    let path = match kind {
        BuildKind::Code      => code::build(dir, &command),
        BuildKind::Inline    => inline::build(dir,name,  &lang, &command),
        BuildKind::Layer     => layer::build(dir, name, &lang, pre, post),
        BuildKind::Library   => library::build(dir, name),
        BuildKind::Slab      => slab::build(dir, name, &lang, pre, post),
        BuildKind::Extension => extension::build(dir, name),
        BuildKind::Image     => image::build(dir, name, &lang, image_kind, spec.images, uri),
        BuildKind::Runtime   => todo!()
    };
    BuildOutput {
        name: u::basename(dir),
        dir: dir.to_string(),
        artifact: path,
        kind: kind,
        runtime: lang.clone()
    }
}

pub fn clean(dir: &str) {
    sh("rm -rf lambda.zip dist __pycache__", dir);
}
