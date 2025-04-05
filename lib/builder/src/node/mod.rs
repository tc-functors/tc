mod layer;
mod code;
mod inline;

use compiler::spec::{BuildKind, LangRuntime};
use super::BuildOutput;
use compiler::Build;
use kit::sh;
use kit as u;

pub fn build(dir: &str, runtime: LangRuntime, name: &str, spec: Build) -> BuildOutput {
        let Build { kind, pre, post, command, .. } = spec;
    let path = match kind {
        BuildKind::Code      => code::build(dir, &command),
        BuildKind::Inline    => inline::build(dir, "inline-deps"),
        BuildKind::Layer     => layer::build(dir, name, &runtime, pre, post),
        BuildKind::Library   => todo!(),
        BuildKind::Extension => todo!(),
        BuildKind::Image     => todo!(),
        BuildKind::Runtime   => todo!(),
        BuildKind::Slab      => todo!()
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
    sh("rm -rf lambda.zip dist __node_modules__", dir);
}
