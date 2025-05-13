mod python;
mod ruby;

use compiler::{LangRuntime, Lang};

pub fn build(dir: &str, langr: &LangRuntime) -> String {

    match langr.to_lang() {
        Lang::Python => python::build(dir),
        Lang::Ruby => ruby::build(dir),
        _ => todo!()
    }
}
