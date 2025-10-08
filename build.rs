use std::process::Command;

fn main() {
    let project_version = "PROJECT_VERSION";
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap();
    let git_tag = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env={project_version}={git_tag}");
}
