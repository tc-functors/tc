mod function;
pub mod llm;
pub mod config;
pub mod provider;
use compiler::LangRuntime;
use kit as u;
use base64::{
    Engine as _,
    engine::general_purpose,
};
use serde::{Deserialize,Serialize};

pub async fn scaffold(
    dir: &str,
    functions: bool,
    llm: bool,
    llm_provider: Option<String>,
    llm_model: Option<String>,
    aws_region: Option<String>,
    aws_profile: Option<String>,
) {
    if llm {
        llm::scaffold(dir, llm_provider, llm_model, aws_region, aws_profile).await;
        visualizer::visualize(&dir);
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

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub desc: String,
    pub code: String,
    pub dot: String
}

pub async fn gen_code_and_diagram(text: &str) -> Response {
    let desc = llm::send(text).await;
    let code = llm::extract_code(&desc);

    let dir = "/tmp/tc-gentopo";
    u::sh(&format!("mkdir -p {}", dir), &u::pwd());
    let topo_file = format!("{}/topology.yml", dir);
    u::write_str(&topo_file, &code);

    let topology = composer::compose(dir, false);
    let dot_str = visualizer::generate_dot(&topology);
    let dot = general_purpose::STANDARD.encode(&dot_str);

    Response {
        desc: desc,
        code: code,
        dot: dot
    }
}
