mod function;
pub mod llm;
use compiler::LangRuntime;
use kit as u;
use base64::{
    Engine as _,
    engine::general_purpose,
};

pub async fn scaffold(dir: &str, functions: bool, llm: bool) {
    if llm {
        llm::scaffold(dir).await;
        visualizer::visualize(&dir, false, "light", vec![]);
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

pub struct Response {
    pub desc: String,
    pub code: String,
    pub diagram: String
}

pub async fn gen_code_and_diagram(text: &str) -> Response {
    let desc = llm::send(text).await;
    let code = llm::extract_code(&desc);

    let dir = "/tmp/tc-gentopo";
    let topo_file = format!("{}/topology.yml", dir);
    u::write_str(&topo_file, &code);

    let topology = composer::compose(dir, false);
    let mermaid_str = visualizer::gen_mermaid(&topology);
    let diagram = general_purpose::STANDARD.encode(&mermaid_str);

    Response {
        desc: desc,
        code: code,
        diagram: diagram
    }
}
