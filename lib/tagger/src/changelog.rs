use crate::git;
use colored::Colorize;
use kit as u;

pub fn find_version(prefix: &str, s: &str) -> Option<String> {
    git::fetch_tags();
    let parts = git::list_tags(prefix);
    for part in parts {
        if part.contains(s) {
            let tag = git::extract_tag(prefix, &part);
            let version = git::extract_version(&tag);
            return Some(version);
        }
    }
    None
}

pub fn list(prefix: &str, verbose: bool) {
    git::fetch_tags();
    let parts = git::list_tags(prefix);
    for part in parts {
        let tag = git::extract_tag(prefix, &part);
        let version = git::extract_version(&tag);
        let commit = u::split_last(&part, ") ");
        if verbose {
            println!("{} {}", version.green(), &commit);
        } else {
            let semver = git::maybe_semver(&version);
            if semver.patch == 0 {
                println!("");
                println!("{}", &version.green());
            }
            println!("{}", &commit);
        }
    }
}

fn parse_version(version: &str) -> (String, String) {
    if version.contains("...") {
        let parts: Vec<&str> = version.split("...").collect();
        let from = u::nth(parts.clone(), 0);
        let to = u::nth(parts.clone(), 1);
        (from, to)
    } else if version.contains("..") {
        let parts: Vec<&str> = version.split("..").collect();
        let from = u::nth(parts.clone(), 0);
        let to = u::nth(parts.clone(), 1);
        (from, to)
    } else {
        (version.to_string(), version.to_string())
    }
}

fn commits(from_tag: &str, to_tag: &str) -> String {
    let from_sha = git::tag_revision(from_tag);
    let to_sha = git::tag_revision(to_tag);
    git::changelog(&from_sha, &to_sha)
}

pub fn between(prefix: &str, versions: Option<String>) {
    match versions {
        Some(version) => {
            git::fetch_tags();
            let (from, to) = parse_version(&version);
            let from_tag = format!("{}-{}", prefix, from);
            let to_tag = format!("{}-{}", prefix, to);
            let cmts = commits(&from_tag, &to_tag);
            println!("{}", cmts);
        }
        _ => println!(""),
    }
}

pub fn generate(from_tag: &str, to_tag: &str) -> String {
    let from_sha = git::tag_revision(from_tag);
    let to_sha = git::tag_revision(to_tag);
    git::changelog(&from_sha, &to_sha)
}
