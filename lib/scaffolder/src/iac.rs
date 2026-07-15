use kit::*;
use composer::Topology;

pub fn generate_prompt(topology: &Topology, iac: &str, out_dir: &str) -> String {
    let iac_str = match iac {
        "tf" | "terraform" => "Terraform code (IAC)",
        "cdk" => "CDK code",
        "boto" => "boto3 code in python",
        "go" => "Go code using go aws-go-sdk",
        "aws-cli" => "aws cli commands",
        "rust" => "rust code using aws-rust-sdk",
        _ => "Terraform code (IAC)"
    };
    let lines = v![
        &topology.to_str(),
        &format!("use this topology definition that contains all the entities as maps and generate the corresponding {} in the directory {}.", iac_str, out_dir)
    ];
    lines.join("\n")

}
