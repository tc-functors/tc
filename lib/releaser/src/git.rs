use kit::{pwd, sh, triml};
use regex::Regex;
use semver::Version;

pub fn extract_version(s: &str) -> String {
    let re: Regex = Regex::new(r"(?:(\d+)\.)?(?:(\d+)\.)?(?:(\d+)\.\d+)").unwrap();
    let matches = re.find(s);
    match matches {
        Some(m) => m.as_str().to_string(),
        _ => "0.0.2".to_string(),
    }
}

pub fn maybe_semver(v: &str) -> Version {
    Version::parse(v).unwrap()
}

pub fn latest_version(prefix: &str) -> String {
    let cmd = format!("git describe --tags --abbrev=0 --match {}-*", prefix);
    let out = sh(&cmd, &pwd());
    if out.contains("fatal: No names found") {
        String::from("0.0.1")
    } else {
        extract_version(&out)
    }
}

pub fn tag_revision(tag: &str) -> String {
    let cmd = format!("git rev-parse {}", tag);
    sh(&cmd, &pwd())
}

pub fn commit_message(tag: &str) -> String {
    let cmd = format!(
        "git show --pretty=format:\"%B\" -n1 --no-patch {} | grep -v Co-authored | grep -iv \"Merge pull\" | head -n1",
        tag
    );
    triml(&sh(&cmd, &pwd())).to_string()
}

pub fn changelog(from_sha: &str, to_sha: &str) -> String {
    let cmd = format!("git log --pretty=\"- %s\" {}...{} .", from_sha, to_sha);
    let out = sh(&cmd, &pwd());
    if out.contains("fatal") {
        String::from("")
    } else {
        out
    }
}

pub fn changelogs(from_sha: &str, to_sha: &str) -> String {
    let cmd = format!("git log --pretty=\"%s\" {}...{} .", from_sha, to_sha);
    let out = sh(&cmd, &pwd());
    if out.contains("fatal") {
        String::from("")
    } else {
        out
    }
}

pub fn fetch_tags() {
    sh("git fetch --tags", &pwd());
}

pub fn create_tag(tag: &str, parent: Option<String>) {
    let cmd = match parent {
        Some(p) => format!("git tag -f {} {}", tag, p),
        None => format!("git tag {}", tag),
    };
    sh(&cmd, &pwd());
}

pub fn create_annotated_tag(tag: &str, parent: Option<String>) {
    let cmd = match parent {
        Some(p) => format!(
            "git -c user.name=tc-releaser -c user.email=tc-releaser@informed.iq tag -a {} {} -m \"{} release\"",
            tag, p, tag
        ),
        None => format!("git tag {}", tag),
    };
    let out = sh(&cmd, &pwd());
    println!("annot: {}", out);
}

pub fn push_tag(tag: &str) {
    let cmd = format!("git push origin {}", tag);
    let out = sh(&cmd, &pwd());
    println!("{}", out);
}

pub fn current_repo() -> String {
    sh(
        "basename -s .git `git config --get remote.origin.url`",
        &pwd(),
    )
}
