mod python;
mod ruby;

use crate::types::BuildStatus;
use compiler::spec::{
    Lang,
    LangRuntime,
};
use composer::Build;

pub fn build(dir: &str, langr: &LangRuntime, bspec: &Build) -> BuildStatus {
    let Build {
        dirs,
        include_deps,
        ..
    } = bspec;

    let path = match langr.to_lang() {
        Lang::Python => python::build(dir, langr, dirs, *include_deps, bspec),
        Lang::Ruby => ruby::build(dir, langr, dirs, *include_deps, bspec),
        _ => todo!(),
    };
    BuildStatus {
        path: path,
        status: true,
        out: String::from(""),
        err: String::from(""),
    }
}
