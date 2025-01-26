mod layer;
mod image;
mod inline;
mod code;
mod library;

use super::BuildOutput;
use compiler::spec::{Kind, LangRuntime};
use compiler::Build;
use kit::sh;
use kit as u;

pub fn build(dir: &str, runtime: LangRuntime, name: &str, spec: Build, trace: bool) -> BuildOutput {

    let Build { kind, pre, post, command, .. } = spec;

    let path = match kind {
        Kind::Code      => code::build(dir, &command),
        Kind::Inline    => inline::build(dir, "inline-deps"),
        Kind::Layer     => layer::build(dir, name, &runtime, pre, post, trace),
        Kind::Library   => library::build(dir, name),
        Kind::Extension => todo!(),
        Kind::Image     => image::build(dir, name),
        Kind::Runtime   => todo!()
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
    sh("rm -f lambda.zip .cpcache", dir);
}
