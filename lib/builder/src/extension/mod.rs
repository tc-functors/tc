mod python;
mod rust;
use crate::types::BuildStatus;
use composer::{
    Lang,
    LangRuntime,
};

pub fn build(dir: &str, name: &str, langr: &LangRuntime) -> BuildStatus {
    let path = match langr.to_lang() {
        Lang::Python => python::build(dir, name),
        Lang::Rust => rust::build(dir),
        _ => String::from(""),
    };
    BuildStatus {
        path: path,
        status: true,
        out: String::from(""),
        err: String::from(""),
    }
}
