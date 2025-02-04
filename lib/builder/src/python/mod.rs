mod layer;
mod image;
mod inline;
mod extension;
mod code;
mod library;

use super::BuildOutput;
use compiler::spec::{BuildKind, LangRuntime};
use compiler::Build;
use kit::sh;
use kit as u;

pub fn build(dir: &str, runtime: LangRuntime, name: &str, spec: Build, trace: bool) -> BuildOutput {

    let Build { kind, pre, post, command, .. } = spec;

    let path = match kind {
        BuildKind::Code      => code::build(dir, &command),
        BuildKind::Inline    => inline::build(dir, "inline-deps"),
        BuildKind::Layer     => layer::build(dir, name, &runtime, pre, post, trace),
        BuildKind::Library   => library::build(dir, name),
        BuildKind::Extension => extension::build(dir, name),
        BuildKind::Image     => image::build(dir, name),
        BuildKind::Runtime   => todo!()
    };
    BuildOutput {
        name: u::basename(dir),
        dir: dir.to_string(),
        zipfile: path,
        kind: kind,
        runtime: runtime
    }
}

pub fn clean(dir: &str) {
    sh("rm -rf lambda.zip dist __pycache__", dir);
}
