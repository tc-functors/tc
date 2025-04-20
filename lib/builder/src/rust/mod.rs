mod extension;
mod inline;
mod layer;
mod image;

use super::BuildOutput;
use compiler::spec::{BuildKind};
use compiler::{Build, Runtime};
use kit as u;

pub fn build(dir: &str, runtime: &Runtime, name: &str, spec: Build) -> BuildOutput {
    let Build { kind, pre, post, .. } = spec;

    let Runtime { lang, .. } = runtime;

    let path = match kind {
        BuildKind::Code      => inline::build(dir),
        BuildKind::Inline    => inline::build(dir),
        BuildKind::Layer     => layer::build(dir, name, &lang, pre, post),
        BuildKind::Extension => extension::build(dir),
        BuildKind::Image     => image::build(dir, name),
        BuildKind::Runtime   => todo!(),
        BuildKind::Library   => todo!(),
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
    u::runcmd_quiet("rm -rf deps.zip build target bootstrap", dir);
}
