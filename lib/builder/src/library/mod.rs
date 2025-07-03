mod python;
mod ruby;

use crate::types::BuildStatus;
use compiler::{
    Lang,
    LangRuntime,
};

pub fn build(dir: &str, langr: &LangRuntime) -> BuildStatus {
    let path = match langr.to_lang() {
        Lang::Python => python::build(dir),
        Lang::Ruby => ruby::build(dir),
        _ => todo!(),
    };
    BuildStatus {
        path: path,
        status: true,
        out: String::from(""),
        err: String::from(""),
    }
}
