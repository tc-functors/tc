mod code;
mod extension;
mod image;
mod inline;
mod layer;
pub mod library;

use super::BuildOutput;
use compiler::{
    Build,
    Runtime,
    spec::{
        function::BuildKind,
        function::LangRuntime,
    },
};
use kit::sh;

pub fn build(
    dir: &str,
    lang: &LangRuntime,
    _runtime: &Runtime,
    name: &str,
    spec: Build,
) -> BuildOutput {
    let Build {
        kind,
        pre,
        post,
        command,
        ..
    } = spec;

    let path = match kind {
        BuildKind::Code => code::build(dir, &command),
        BuildKind::Inline => inline::build(dir, "inline-deps", &command),
        BuildKind::Layer => layer::build(dir, name, &lang, pre, post),
        BuildKind::Library => library::build(dir),
        BuildKind::Extension => extension::build(dir, name),
        BuildKind::Image => image::build(dir, name),
        BuildKind::Runtime => todo!(),
        BuildKind::Slab => todo!(),
    };

    BuildOutput {
        name: name.to_string(),
        dir: dir.to_string(),
        artifact: path,
        kind: kind,
        runtime: lang.clone(),
    }
}

pub fn clean(dir: &str) {
    sh("rm -f lambda.zip", dir);
}
