mod layer;
mod image;
mod inline;
mod extension;
mod code;
mod library;

use super::BuildOutput;
use compiler::spec::{BuildKind};
use compiler::{Build, Runtime};
use kit::sh;
use kit as u;

pub fn build(dir: &str, runtime: &Runtime, name: &str, spec: Build) -> BuildOutput {

    let Build { kind, pre, post, command, .. } = spec;
    let Runtime { lang, .. } = runtime;

    let path = match kind {
        BuildKind::Code      => code::build(dir, &command),
        BuildKind::Inline    => inline::build(dir, "inline-deps", &command),
        BuildKind::Layer     => layer::build(dir, name, &lang, pre, post),
        BuildKind::Library   => library::build(dir),
        BuildKind::Extension => extension::build(dir, name),
        BuildKind::Image     => image::build(dir, name),
        BuildKind::Runtime   => todo!(),
        BuildKind::Slab      => todo!()
    };

    BuildOutput {
        name: format!("{}-{}", name, u::basename(dir)),
        dir: dir.to_string(),
        artifact: path,
        kind: kind,
        runtime: lang.clone()
    }
}

pub fn clean(dir: &str) {
    sh("rm -f lambda.zip", dir);
}
