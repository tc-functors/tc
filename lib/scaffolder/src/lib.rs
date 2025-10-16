mod function;
mod llm;
use compiler::LangRuntime;

use kit as u;

pub async fn scaffold(dir: &str, functions: bool, llm: bool) {
    if llm {
        llm::scaffold(dir).await;
        let topology = composer::compose(dir, false);
        composer::display::print_graph(&topology);
        println!("Rendering SVG in browser...");
    } else if functions {
        let topology = composer::compose(dir, false);
        for (_, f) in topology.functions {
            u::sh(&format!("mkdir -p {}", &f.dir), dir);
            function::write_handler(&f.dir, &LangRuntime::Python311);
        }
    } else {
        function::scaffold();
    }
}
