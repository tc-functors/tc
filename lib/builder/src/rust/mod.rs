mod extension;
mod inline;
mod layer;
mod image;

use super::BuildOutput;
use compiler::spec::{Kind, LangRuntime};
use compiler::Build;
use kit as u;

pub fn build(dir: &str, runtime: LangRuntime, name: &str, spec: Build, trace: bool) -> BuildOutput {
    let Build { kind, pre, post, .. } = spec;

    let path = match kind {
        Kind::Code      => inline::build(dir, trace),
        Kind::Inline    => inline::build(dir, trace),
        Kind::Layer     => layer::build(dir, name, &runtime, pre, post, trace),
        Kind::Extension => extension::build(dir),
        Kind::Image     => image::build(dir, name),
        Kind::Runtime   => todo!(),
        Kind::Library   => todo!()
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
