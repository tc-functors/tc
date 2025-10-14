mod function;
mod llm;

use kit as u;

pub async fn scaffold(dir: &str, functions: bool, llm: bool) {
    if functions {
        let topology = composer::compose(dir, false);
        for (_, f) in topology.functions {
            u::sh(&format!("mkdir -p {}", &f.dir), dir);
        }
    } else if llm {
        llm::scaffold(dir).await;
    } else {
        function::scaffold();
    }
}
