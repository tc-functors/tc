mod code;
mod inline;
mod layer;

use super::BuildOutput;
use compiler::{
    Build,
    Runtime,
    spec::{
        BuildKind,
        LangRuntime,
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
        BuildKind::Inline => inline::build(dir, "inline-deps"),
        BuildKind::Layer => layer::build(dir, name, &lang, pre, post),
        BuildKind::Library => todo!(),
        BuildKind::Extension => todo!(),
        BuildKind::Image => todo!(),
        BuildKind::Runtime => todo!(),
        BuildKind::Slab => todo!(),
    };
    BuildOutput {
        name: String::from(name),
        dir: dir.to_string(),
        artifact: path,
        kind: kind,
        runtime: lang.clone(),
    }
}

pub fn clean(dir: &str) {
    sh("rm -rf lambda.zip dist __node_modules__", dir);
}
