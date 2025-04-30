use kit::{
    pwd,
    sh,
};
use regex::Regex;

fn extract_version(s: &str) -> String {
    let re: Regex = Regex::new(r"(?:(\d+)\.)?(?:(\d+)\.)?(?:(\d+)\.\d+)").unwrap();
    let matches = re.find(s);
    match matches {
        Some(m) => m.as_str().to_string(),
        _ => "0.0.2".to_string(),
    }
}

pub fn current_semver(prefix: &str) -> String {
    let cmd = format!(
        "git describe --match {}-* --tags $(git log -n1 --pretty='%h')",
        prefix
    );
    let out = sh(&cmd, &pwd());
    if out.contains("fatal") {
        String::from("0.0.1")
    } else {
        extract_version(&out)
    }
}

pub fn branch_name() -> String {
    sh("git rev-parse --abbrev-ref HEAD", &pwd())
}
