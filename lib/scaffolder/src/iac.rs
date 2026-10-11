use composer::Topology;
use provider::{
    Auth,
    aws::bedrock,
};
use base64::{
    Engine as _,
    engine::general_purpose,
};
use kit::*;
use kit as u;
use std::collections::HashMap;
use llm_toolkit::extract::FlexibleExtractor;

const DEFAULT_BEDROCK_MODEL: &str = "us.anthropic.claude-sonnet-4-5-20250929-v1:0";

fn process_response(response: &str) {
    let extractor = FlexibleExtractor::new();

    let json = extractor.extract(response).unwrap();
    let kvs: HashMap<String, String> = serde_json::from_str(&json).unwrap();

    for (k, v) in kvs {
        let bytes = general_purpose::STANDARD.decode(&v).unwrap();
        let code = String::from_utf8_lossy(&bytes);
        let path = std::path::Path::new(&k);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        println!("Writing file {}", &k);
        u::write_str(&k, &code);
    }
}

pub fn make_prompt(skills: Option<String>, topology: &Topology, iac: &str, out_dir: &str) -> String {
    let f = match skills {
        Some(s) => s,
        None => "skills/iac.md".to_string()
    };
    if u::file_exists(&f) {
        u::slurp(&f)
    } else {
        let iac_str = match iac {
        "tf" | "terraform" => "Terraform code (IAC)",
        "cdk" => "CDK code",
        "boto" | "python" => "boto3 code in python",
        "go" => "Go code using go aws-go-sdk",
        "aws-cli" => "aws cli commands",
        "rust" => "rust code using aws-rust-sdk",
        _ => "Terraform code (IAC)",
        };
        let lines = v![
            &topology.to_str(),
            &format!(
                "use this templated topology definition that contains all the entities as maps and generate the corresponding {} in the directory {}. Generate markdown response with a JSON blob containing key as file name and value as the code as base64 string. Generate a README file in the {} directory with instructions to run the code",
                iac_str, out_dir, out_dir
            )
        ];
        lines.join("\n")
    }
}

pub async fn scaffold(
    auth: &Auth,
    skills: Option<String>,
    model: Option<String>,
    topology: &Topology,
    iac: Option<String>,
    out_dir: Option<String>
) {
    let dir = u::maybe_string(out_dir, "iac");
    let iac = u::maybe_string(iac, "tf");

    let model = u::maybe_string(model, &DEFAULT_BEDROCK_MODEL);
    println!("Using bedrock model {}", &model);

    let client = bedrock::make_client(auth).await;
    let prompt = make_prompt(skills, topology, &iac, &dir);
    let response = bedrock::send(&client, &prompt, &model).await;
    process_response(&response);
}
