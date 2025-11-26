mod function;
mod prompt;
mod anthropic;
use compiler::LangRuntime;
use kit as u;
use provider::Auth;
use provider::aws::bedrock;
use inquire::Text;

pub const DEFAULT_BEDROCK_MODEL: &str = "us.anthropic.claude-sonnet-4-5-20250929-v1:0";
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-5-20250929";

fn resolve_prompt(given_text: &str) -> String {
    if given_text == "-" || given_text == "default" {
        let desc = Text::new("Architecture Description:").prompt();
        let text = &desc.unwrap();
        prompt::default(text)
    } else {
        given_text.to_string()
    }
}

fn process_response(dir: &str, response: &str) {
    println!("Generating topology.yml...");
    let code = llm_toolkit::extract_markdown_block_with_lang(&response, "yaml").unwrap();
    let topo_file = format!("{}/topology.yml", dir);
    u::write_str(&topo_file, &code);
}

pub async fn scaffold_llm_bedrock(auth: &Auth, dir: &str, given_text: &str, model: Option<String>) {
    let model = u::maybe_string(model, DEFAULT_BEDROCK_MODEL);
    println!("Using bedrock model {}", &model);
    let client = bedrock::make_client(auth).await;
    let prompt = resolve_prompt(given_text);
    let response = bedrock::send(&client, &prompt, &model).await;
    process_response(dir, &response);
}

pub async fn scaffold_llm_anthropic(dir: &str, given_text: &str, model: Option<String>) {
    let model = u::maybe_string(model, DEFAULT_ANTHROPIC_MODEL);
    println!("Using anthropic model {}", &model);
    let prompt = resolve_prompt(given_text);
    let response = anthropic::send(&prompt, &model).await;
    process_response(dir, &response);
}


pub fn scaffold_functions(dir: &str) {
    let topology = composer::compose(dir, false);
    for (_, f) in topology.functions {
        u::sh(&format!("mkdir -p {}", &f.dir), dir);
        function::write_handler(&f.dir, &LangRuntime::Python311);
    }
}

pub fn scaffold_function() {
    function::scaffold();
}
