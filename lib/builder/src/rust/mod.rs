mod extension;
mod inline;
mod layer;
mod image;

use super::BuildOutput;
use compiler::spec::{BuildKind, LangRuntime};
use compiler::Build;
use kit as u;

pub fn build(dir: &str, runtime: LangRuntime, name: &str, spec: Build) -> BuildOutput {
    let Build { kind, pre, post, .. } = spec;

    let path = match kind {
        BuildKind::Code      => inline::build(dir),
        BuildKind::Inline    => inline::build(dir),
        BuildKind::Layer     => layer::build(dir, name, &runtime, pre, post),
        BuildKind::Extension => extension::build(dir),
        BuildKind::Image     => image::build(dir, name),
        BuildKind::Runtime   => todo!(),
        BuildKind::Library   => todo!()
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
    u::runcmd_quiet("rm -rf deps.zip build target bootstrap", dir);
}
