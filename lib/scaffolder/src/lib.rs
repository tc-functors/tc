pub mod function;
pub mod iac;
pub mod topology;
use compiler::LangRuntime;
use kit as u;

pub fn scaffold_functions(dir: &str) {
    let topology = composer::compose(dir, false);
    for (_, f) in topology.functions {
        u::sh(&format!("mkdir -p {}", &f.dir), dir);
        function::write_handler(&f.dir, &LangRuntime::Python312);
    }
}
