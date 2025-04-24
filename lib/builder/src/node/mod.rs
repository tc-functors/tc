mod layer;
mod code;
mod inline;

use compiler::spec::{BuildKind, LangRuntime};
use super::BuildOutput;
use compiler::{Build, Runtime};
use kit::sh;
use kit as u;

pub fn build(
    dir: &str,
    lang: &LangRuntime,
    _runtime: &Runtime,
    name: &str,
    spec: Build
) -> BuildOutput {

    let Build { kind, pre, post, command, .. } = spec;

    let path = match kind {
        BuildKind::Code      => code::build(dir, &command),
        BuildKind::Inline    => inline::build(dir, "inline-deps"),
        BuildKind::Layer     => layer::build(dir, name, &lang, pre, post),
        BuildKind::Library   => todo!(),
        BuildKind::Extension => todo!(),
        BuildKind::Image     => todo!(),
        BuildKind::Runtime   => todo!(),
        BuildKind::Slab      => todo!()
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
    sh("rm -rf lambda.zip dist __node_modules__", dir);
}
