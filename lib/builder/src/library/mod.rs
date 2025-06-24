mod python;
mod ruby;

use compiler::{
    Lang,
    LangRuntime,
};
use crate::types::BuildStatus;

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
        err: String::from("")
    }
}
